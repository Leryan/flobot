use crate::joke::{Error, Random, Result};
use chrono::{DateTime, Duration, Local};
use flobot_lib::client::Notifier;
use flobot_lib::task::{self, ExecIn, Task};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::result::Result as StdResult;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration as StdDuration;
use url::form_urlencoded;
use uuid::Uuid;

fn dfs(secs: u64) -> Duration {
    Duration::from_std(StdDuration::from_secs(secs)).unwrap()
}

#[derive(Deserialize)]
pub struct TokenV5 {
    #[serde(skip)]
    pub expired_after: Option<DateTime<Local>>,
    #[serde(skip)]
    pub refresh_token_expired_after: Option<DateTime<Local>>,
    pub access_token: String,
    pub refresh_token: String,
    pub response_type: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token_expires_in: u64,
    pub scope: String,
}

#[derive(Deserialize)]
pub struct TokenV3Data {
    pub access_token: String,
    pub expires_at: i64,
    pub consumer_id: u64,
    pub token_type: String,
    pub authorized: bool,
    pub scope: String,
}

#[derive(Deserialize)]
pub struct TokenV3 {
    pub status: String,
    pub message: String,
    pub code: i64,
    pub data: TokenV3Data,
}

impl TokenV5 {
    pub fn compute_refresh(&mut self, rt: bool) {
        // keep a margin of 10%
        self.expired_after = Some(Local::now() + dfs(self.expires_in));
        if rt {
            self.refresh_token_expired_after =
                Some(Local::now() + dfs(self.refresh_token_expires_in));
        }
    }
}

pub struct Pinterest<N: Notifier> {
    token: Arc<RwLock<Option<TokenV3>>>,
    posted_auth_link: AtomicBool,
    state: String,
    redirect: String,
    board_id: String,
    client_id: String,
    client_secret: String,
    notifier: Mutex<N>,
}

impl<N: Notifier> Pinterest<N> {
    pub fn new(
        client_id: &str,
        client_secret: &str,
        redirect: &str,
        board_id: &str,
        notifier: N,
    ) -> Self {
        Self {
            token: Arc::new(RwLock::new(None)),
            state: Uuid::new_v4().to_string(),
            redirect: redirect.to_string(),
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            board_id: board_id.to_string(),
            notifier: Mutex::new(notifier),
            posted_auth_link: AtomicBool::new(false),
        }
    }

    pub fn auth_url(&self) -> String {
        let params = form_urlencoded::Serializer::new(String::new())
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", &self.redirect)
            .append_pair("response_type", "code")
            .append_pair("scope", "read_boards,read_pins")
            .append_pair("state", &self.state)
            .finish();

        format!("https://www.pinterest.com/oauth/?{}", params)
    }

    fn oauth_authorization(&self) -> String {
        let cid_cs = format!("{}:{}", self.client_id, self.client_secret);
        format!("Bearer {}", base64::encode(cid_cs))
    }

    pub fn authenticate(&self, code: &str, state: &str) -> bool {
        if state != self.state || code == "" {
            return false;
        }

        let mut form = std::collections::HashMap::new();
        form.insert("code", code);
        form.insert("redirect_uri", &self.redirect);
        form.insert("grant_type", "authorization_code");

        let res = Client::new()
            .post("https://api.pinterest.com/v3/oauth/access_token")
            .header(reqwest::header::AUTHORIZATION, self.oauth_authorization())
            .form(&form)
            .send();

        if res.is_err() {
            println!("pinterest: error on authentication: {:?}", res);
            return false;
        }

        let res = res.unwrap().json::<TokenV3>();
        if let Ok(token) = res {
            let mut guard = self.token.write().unwrap();
            // V5
            //let mut token = token;
            //token.compute_refresh(true);
            println!("pinterest access token: {}", &token.data.access_token);
            println!("pinterest scope: {}", &token.data.scope);
            *guard = Some(token);
            return true;
        }

        println!("pinterest authenticate error: {:?}", res.err());

        false
    }
}

impl<N: Notifier> Random for Pinterest<N> {
    fn random(&self, _team_id: &str) -> Result {
        if let Some(token) = &((*self.token.read().unwrap()).as_ref()) {
            let at = token.data.access_token.clone();
            let url =
                format!("https://api.pinterest.com/v3/boards/{}/pins", self.board_id);

            let val = Client::new()
                .get(url)
                .bearer_auth(&at)
                .send()?
                .json::<serde_json::Value>()?;

            println!("bord list: {:?}", val);

            if val.get("code").is_none() {
                let items: &Vec<serde_json::Value> =
                    val.get("items").unwrap().as_array().unwrap();
                let pin_id = items[0].get("id").unwrap().as_str().unwrap();

                let url = format!("https://api.pinterest.com/v3/pins/{}", pin_id);
                let val = Client::new()
                    .get(url)
                    .bearer_auth(&at)
                    .send()?
                    .json::<serde_json::Value>()?;

                println!("pin: {:?}", val);
            }
        }

        Err(Error::NoData("no data".to_string()))
    }
}

impl<N: Notifier> Task for Pinterest<N> {
    fn name(&self) -> String {
        "pinterest.token".to_string()
    }

    fn init_exec(&self, _now: task::Now) -> ExecIn {
        std::time::Duration::from_secs(0)
    }

    fn exec(&self, now: task::Now) -> StdResult<ExecIn, task::Error> {
        if !self.posted_auth_link.load(Ordering::Relaxed) {
            if let Ok(_) = self
                .notifier
                .lock()
                .unwrap()
                .required_action(&self.auth_url())
            {
                self.posted_auth_link.store(true, Ordering::Relaxed);
            }
        } else {
            let guard = self.token.read().unwrap();
            if let Some(ref token) = *guard {
                if token.data.expires_at <= (now - dfs(3600)).timestamp() {
                    self.posted_auth_link.store(false, Ordering::Relaxed);
                }
            }
        }

        Ok(std::time::Duration::from_secs(60))
    }
}

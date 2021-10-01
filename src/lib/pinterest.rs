use crate::client::Notifier;
use crate::joke::{Error, Random, Result};
use crate::task::{ExecIn, Task};
use chrono::{DateTime, Duration, Local};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::result::Result as StdResult;
use std::sync::{Arc, RwLock};
use std::time::Duration as StdDuration;
use url::form_urlencoded;
use uuid::Uuid;

fn dfs(secs: u64) -> Duration {
    Duration::from_std(StdDuration::from_secs(secs)).unwrap()
}

#[derive(Deserialize)]
pub struct Token {
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

impl Token {
    pub fn compute_refresh(&mut self, rt: bool) {
        // keep a margin of 10%
        self.expired_after = Some(Local::now() + dfs(self.expires_in));
        if rt {
            self.refresh_token_expired_after = Some(Local::now() + dfs(self.refresh_token_expires_in));
        }
    }
}
/*

   if let Some(hh) = make_pinterest(mm_client.clone(), &mut taskrunner, &mut jokeproviders) {
        thread::spawn(move || loop {
            // survive crashes from webserver
            let hh = hh.clone();
            let r = thread::spawn(move || {
                let mut server = ss::Server::new(hh);
                server.dont_serve_static_files();
                println!("launched webserver on localhost:6799");
                server.listen("localhost", "6799");
            })
            .join();
            println!("webserver thread loop returned: {:?}", r);
        });
    }

fn make_pinterest(mm_client: Mattermost, taskrunner: &mut SequentialTaskRunner, jokeproviders: &mut Vec<flobot::db::remote::blague::Remote>) {
    // PINTEREST
    if let (Ok(client_id), Ok(client_secret), Ok(board_id), Ok(redirect)) = (
        env::var("CLIENT_ID"),
        env::var("CLIENT_SECRET"),
        env::var("BOARD_ID"),
        env::var("REDIRECT"),
    ) {
        println!("loading pinterest");
        let pinterest = Arc::new(flobot::pinterest::Pinterest::new(
            &client_id,
            &client_secret,
            &redirect,
            &board_id,
            mm_client.clone(),
        ));

        taskrunner.add(pinterest.clone());

        jokeproviders.clear();
        jokeproviders.push(pinterest.clone());

        let pinterest_http = pinterest.clone();
        return Some(
            move |request: ss::Request<Vec<u8>>, mut response: ss::Builder| -> ss::ResponseResult {
                let furl = format!("http://localhost{}", request.uri());

                let mut code = "".to_string();
                let mut state = "".to_string();

                if let Ok(url) = url::Url::parse(&furl) {
                    for qp in url.query_pairs() {
                        if qp.0 == "code" {
                            code = qp.1.to_string();
                        } else if qp.0 == "state" {
                            state = qp.1.to_string();
                        }
                    }
                }

                println!("pinterest: got code {}", code);
                if pinterest_http.authenticate(&code, &state) {
                    println!("authenticated!");
                    return Ok(response.status(StatusCode::OK).body("Authenticated!".as_bytes().to_vec())?);
                }

                println!("pinterest: failed to authenticate");

                Ok(response
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body("NOT AUTHENTICATED".as_bytes().to_vec())?)
            },
        );
    }

    None
}
 */
pub struct Pinterest<N> {
    token: Arc<RwLock<Option<Token>>>,
    state: String,
    redirect: String,
    board_id: String,
    client_id: String,
    client_secret: String,
    notifier: N,
}

impl<N: crate::client::Notifier> Pinterest<N> {
    pub fn new(client_id: &str, client_secret: &str, redirect: &str, board_id: &str, notifier: N) -> Self {
        Self {
            token: Arc::new(RwLock::new(None)),
            state: Uuid::new_v4().to_string(),
            redirect: redirect.to_string(),
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            board_id: board_id.to_string(),
            notifier: notifier,
        }
    }

    pub fn auth_url(&self) -> String {
        let params = form_urlencoded::Serializer::new(String::new())
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", &self.redirect)
            .append_pair("response_type", "code")
            .append_pair("scope", "boards:read,pins:read")
            .append_pair("state", &self.state)
            .finish();

        format!("https://www.pinterest.com/oauth/?{}", params)
    }

    fn authorization(&self) -> String {
        format!(
            "Basic {}",
            base64::encode(format!("{}:{}", self.client_id, self.client_secret).as_bytes())
        )
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
            .post("https://api.pinterest.com/v5/oauth/token")
            .header(reqwest::header::AUTHORIZATION, self.authorization())
            .form(&form)
            .send();

        if res.is_err() {
            println!("pinterest: error on authentication: {:?}", res);
            return false;
        }

        let res = res.unwrap().json::<Token>();
        if let Ok(token) = res {
            let mut guard = self.token.write().unwrap();
            let mut token = token;
            token.compute_refresh(true);
            *guard = Some(token);

            return true;
        }

        println!("pinterest authenticate error: {:?}", res.err());

        false
    }

    pub fn reauthenticate(&self) -> bool {
        // mutex -> guard -> ok guard -> refcell -> borrow -> option<t> -> option<&t> -> unwrap -> field -> clone
        let rt = (*self.token.read().unwrap()).as_ref().unwrap().refresh_token.clone();
        let mut form = std::collections::HashMap::new();
        form.insert("grant_type", "refresh_token");
        form.insert("refresh_token", &rt);
        form.insert("scope", "boards:read,pins:read");

        let res = Client::new()
            .post("https://api.pinterest.com/v5/oauth/token")
            .header(reqwest::header::AUTHORIZATION, self.authorization())
            .form(&form)
            .send();

        if res.is_err() {
            return false;
        }

        if let Ok(token) = res.unwrap().json::<serde_json::Value>() {
            let at = token.get("access_token").unwrap().as_str().unwrap();
            let ei = token.get("expires_in").unwrap().as_u64().unwrap();

            let mut guard = self.token.write().unwrap();
            let token = (*guard).as_mut().unwrap();
            token.expires_in = ei;
            token.access_token = at.to_string();
            token.compute_refresh(false);

            return true;
        }

        false
    }
}

impl<N> Random for Pinterest<N> {
    fn random(&self, _team_id: &str) -> Result {
        println!("pinterest: !blague called");
        if let Some(token) = &((*self.token.read().unwrap()).as_ref()) {
            let at = token.access_token.clone();
            let url = format!("https://api.pinterest.com/v5/boards/{}/pins", self.board_id);

            let val = Client::new().get(url).bearer_auth(&at).send()?.json::<serde_json::Value>()?;

            println!("bord list: {:?}", val);

            if val.get("code").is_none() {
                let items: &Vec<serde_json::Value> = val.get("items").unwrap().as_array().unwrap();
                let pin_id = items[0].get("id").unwrap().as_str().unwrap();

                let url = format!("https://api.pinterest.com/v5/pins/{}", pin_id);
                let val = Client::new().get(url).bearer_auth(&at).send()?.json::<serde_json::Value>()?;

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

    fn init_exec(&self, _now: crate::task::Now) -> ExecIn {
        std::time::Duration::from_secs(0)
    }

    fn exec(&self, now: crate::task::Now) -> StdResult<ExecIn, crate::task::Error> {
        let mut do_refresh = false;

        if (*self.token.read().unwrap()).is_none() {
            let _ = self.notifier.required_action(&self.auth_url());
        } else {
            let guard = self.token.read().unwrap();
            if let Some(ref token) = *guard {
                if token.refresh_token_expired_after.unwrap() <= now - dfs(3600) {
                    let _ = self.notifier.required_action(&self.auth_url());
                } else if token.expired_after.unwrap() <= now - dfs(3600) {
                    do_refresh = true;
                }
            }
        }

        if do_refresh {
            self.reauthenticate();
        }

        Ok(std::time::Duration::from_secs(60))
    }
}

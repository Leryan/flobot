mod models;

use crate::client::*;
use crate::conf::Conf;
use crate::mattermost::models::*;
use crate::models::*;
use crossbeam::crossbeam_channel::Sender;
use reqwest;
use serde::Serialize;
use serde_json::json;
use ws::Result as WSResult;
use ws::Sender as WSSender;
use ws::{connect, CloseCode, Handler, Handshake, Message};

struct MattermostWS {
    out: WSSender,
    send: Sender<GenericEvent>,
    token: String,
    seq: u64,
}

impl Handler for MattermostWS {
    fn on_open(&mut self, _: Handshake) -> WSResult<()> {
        self.seq += 1;
        let auth = json!({
            "action": "authentication_challenge",
            "data": {"token": self.token.clone()},
            "seq": self.seq,
        });
        self.out.send(Message::Text(auth.to_string()))
    }

    fn on_message(&mut self, msg: Message) -> WSResult<()> {
        let txt = msg.as_text().unwrap();
        let event: MetaEvent = match serde_json::from_str(txt) {
            Ok(v) => v,
            Err(_e) => MetaEvent::Unsupported(msg.to_string()),
        };

        match self.send.send(event.into()) {
            Err(e) => self.out.close_with_reason(CloseCode::Error, e.to_string()),
            Ok(()) => Ok(()),
        }
    }
}

pub struct Mattermost {
    cfg: Conf,
    user_id: String,
    debug: bool,
}

impl Mattermost {
    pub fn new(cfg: Conf) -> Self {
        Mattermost {
            cfg: cfg,
            user_id: String::new(),
            debug: false,
        }
    }
}

impl EventClient for Mattermost {
    fn listen(&self, sender: Sender<GenericEvent>) {
        let mut url = self.cfg.ws_url.clone();
        url.push_str("/api/v4/websocket");
        connect(url, |out| MattermostWS {
            out: out,
            send: sender.clone(),
            token: self.cfg.token.clone(),
            seq: 0,
        })
        .unwrap()
    }

    fn client(&self) -> Box<dyn Client> {
        unimplemented!()
    }
}

#[derive(Serialize)]
struct Metadata {}

#[derive(Serialize)]
struct Props {}

#[derive(Serialize)]
struct Post<'a> {
    channel_id: String,
    create_at: u64,
    file_ids: Vec<String>,
    message: &'a str,
    metadata: Metadata,
    props: Props,
    update_at: u64,
    user_id: String,
    root_id: Option<String>,
    parent_id: Option<String>,
}

#[derive(Serialize)]
struct Reaction {
    user_id: String,
    post_id: String,
    emoji_name: String,
}

impl Mattermost {
    fn url(&self, add: &str) -> String {
        let mut url = self.cfg.api_url.clone();
        url.push_str(add);
        url
    }

    fn response_result(&self, r: reqwest::Result<reqwest::blocking::Response>) -> Result<()> {
        match r {
            Ok(r) => {
                if self.debug {
                    println!(
                        "{:?} {:?}",
                        r.status(),
                        r.text().unwrap_or(String::from("no text"))
                    )
                }
            }
            Err(e) => println!("{:?}", e),
        };
        Ok(())
    }

    pub fn toggle_debug(&mut self) {
        self.debug = !self.debug
    }
}

impl Client for Mattermost {
    fn set_my_user_id(&mut self, user_id: &str) -> Result<()> {
        self.user_id = String::from(user_id);
        Ok(())
    }

    fn send_post(&self, post: GenericPost) -> Result<()> {
        let c = reqwest::blocking::Client::new();
        let mmpost = Post {
            channel_id: post.channel_id.clone(),
            create_at: 0,
            file_ids: vec![],
            message: &post.message,
            metadata: Metadata {},
            props: Props {},
            update_at: 0,
            user_id: self.user_id.clone(),
            parent_id: None,
            root_id: None,
        };
        self.response_result(
            c.post(&self.url("/posts"))
                .bearer_auth(self.cfg.token.clone())
                .json(&mmpost)
                .send(),
        )
    }

    fn send_message(&self, mut post: GenericPost, message: &str) -> Result<()> {
        post.message = message.to_string();
        self.send_post(post)
    }

    fn send_reaction(&self, post: GenericPost, reaction: &str) -> Result<()> {
        let c = reqwest::blocking::Client::new();
        let reaction = Reaction {
            user_id: self.user_id.clone(),
            post_id: post.id.clone(),
            emoji_name: String::from(reaction),
        };
        self.response_result(
            c.post(&self.url("/reactions"))
                .bearer_auth(self.cfg.token.clone())
                .json(&reaction)
                .send(),
        )
    }

    fn send_reply(&self, post: GenericPost, message: &str) -> Result<()> {
        let c = reqwest::blocking::Client::new();
        let mmpost = Post {
            channel_id: post.channel_id.clone(),
            create_at: 0,
            file_ids: vec![],
            message: message,
            metadata: Metadata {},
            props: Props {},
            update_at: 0,
            user_id: self.user_id.clone(),
            parent_id: Some(post.id.clone()),
            root_id: Some(post.id.clone()),
        };
        self.response_result(
            c.post(&self.url("/posts"))
                .bearer_auth(self.cfg.token.clone())
                .json(&mmpost)
                .send(),
        )
    }

    fn send_trigger_list(&self, triggers: Vec<Trigger>, from: GenericPost) -> Result<()> {
        let mut l = String::from(format!("Ya {:?} triggers.\n", triggers.len()));
        let mut count = 0;

        for trigger in triggers {
            count += 1;
            if trigger.emoji.is_some() {
                l.push_str(&format!(
                    " * `{}`: :{}:\n",
                    trigger.triggered_by,
                    trigger.emoji.unwrap()
                ));
            } else {
                l.push_str(&format!(
                    " * `{}`: {}\n",
                    trigger.triggered_by,
                    trigger.text_.unwrap()
                ));
            }

            if count == 20 {
                self.send_message(from.clone(), &l)?;
                count = 0;
                l = String::new();
            }
        }

        if count > 0 {
            self.send_message(from, &l)?;
        }

        Ok(())
    }

    fn notify_startup(&self) -> Result<()> {
        let mut post = GenericPost::with_message("jsuilà");
        post.channel_id = self.cfg.debug_channel.clone();
        self.send_post(post)
    }

    fn edit_post_message(&self, post_id: &str, message: &str) -> Result<()> {
        let edit = PostEdit {
            message: Some(message),
            file_ids: None,
        };

        let c = reqwest::blocking::Client::new();
        self.response_result(
            c.put(&self.url(&format!("/posts/{}/patch", post_id)))
                .bearer_auth(self.cfg.token.clone())
                .json(&edit)
                .send(),
        )
    }

    fn unimplemented(&self, post: GenericPost) -> Result<()> {
        self.send_reply(post, "spô encore prêt chéri·e :3 :trollface:")
    }

    fn debug(&self, message: &str) -> Result<()> {
        let mut post = GenericPost::with_message(message);
        post.channel_id = self.cfg.debug_channel.clone();
        self.send_post(post)
    }
}

use crate::client::Client;
use crate::conf::Conf;
use crate::models::mattermost::MetaEvent;
use crate::models::Event;
use crossbeam::crossbeam_channel::Sender;
use serde::{Deserialize, Serialize};
use serde_json::json;
use ws::Sender as WSSender;
use ws::{connect, CloseCode, Handler, Handshake, Message, Result};

#[derive(Serialize, Deserialize)]
struct Auth {
    token: String,
}

struct MattermostWS {
    out: WSSender,
    send: Sender<Event>,
    token: String,
    seq: u64,
}

impl Handler for MattermostWS {
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        self.seq += 1;
        let auth = json!({
            "action": "authentication_challenge",
            "data": {"token": self.token.clone()},
            "seq": self.seq,
        });
        self.out.send(Message::Text(auth.to_string()))
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
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
}

impl Mattermost {
    pub fn new(cfg: Conf) -> Self {
        Mattermost { cfg: cfg }
    }
}

impl Client for Mattermost {
    fn listen(&self, sender: Sender<Event>) {
        let mut url = self.cfg.ws_url.clone();
        url.push_str("/api/v4/websocket");
        connect(url.as_str(), |out| MattermostWS {
            out: out,
            send: sender.clone(),
            token: self.cfg.token.clone(),
            seq: 0,
        })
        .unwrap()
    }
}

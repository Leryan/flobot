use super::models::MetaEvent;
use crate::client::EventClient;
use crate::models::*;
use crossbeam::crossbeam_channel::Sender as ChannelSender;
use serde_json::json;
use ws::Result as WSResult;
use ws::Sender as WSSender;
use ws::{connect, CloseCode, Handler, Handshake, Message};

struct MattermostWS {
    out: WSSender,
    send: ChannelSender<GenericEvent>,
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

impl EventClient for super::client::Mattermost {
    fn listen(&self, sender: ChannelSender<GenericEvent>) {
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
}

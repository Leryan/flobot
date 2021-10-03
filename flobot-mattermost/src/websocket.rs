use super::models::MetaEvent;
use flobot_lib::models::Event;
use serde_json::json;
use std::sync::mpsc::Sender as ChannelSender;
use ws::{connect, CloseCode, Handler, Handshake, Message, Sender};

type Result = ws::Result<()>;

struct MattermostWS {
    out: Sender,
    send: ChannelSender<Event>,
    token: String,
    seq: u64,
}

impl Handler for MattermostWS {
    fn on_open(&mut self, _: Handshake) -> Result {
        self.seq += 1;
        let auth = json!({
            "action": "authentication_challenge",
            "data": {"token": self.token.clone()},
            "seq": self.seq,
        });
        let res = self.out.send(Message::Text(auth.to_string()));

        if res.is_ok() {
            println!("websocket connected!");
        }

        res
    }

    fn on_message(&mut self, msg: Message) -> Result {
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

impl super::client::Mattermost {
    pub fn listen(&self, sender: ChannelSender<Event>) {
        let mut url = self.cfg.ws_url.clone();
        url.push_str("/api/v4/websocket");

        let reco_time = std::time::Duration::from_secs(5);
        let mut retry = true;

        while retry {
            if let Err(e) = connect(url.clone(), |out| MattermostWS {
                out,
                send: sender.clone(),
                token: self.cfg.token.clone(),
                seq: 0,
            }) {
                match e.kind {
                    ws::ErrorKind::Io(details) => {
                        println!("websocket io error: {:?}", details);
                    }
                    e => {
                        println!(
                            "websocket disconnected with unrecoverable error: {:?}",
                            e
                        );
                        retry = false;
                    }
                }
            }

            if retry {
                println!(
                    "websocket returned, retrying in {} seconds",
                    reco_time.as_secs()
                );
                std::thread::sleep(reco_time);
            } else {
                return;
            }
        }
    }
}

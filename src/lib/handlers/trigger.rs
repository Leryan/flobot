use crate::client;
use crate::db;
use crate::db::tempo::Tempo;
use crate::handlers::{Handler, Result};
use crate::models::GenericPost;
use regex::Regex;
use std::rc::Rc;
use std::time::Duration;

pub struct Trigger<C, E> {
    db: Rc<E>,
    client: Rc<C>,
    match_list: Regex,
    match_del: Regex,
    match_text: Regex,
    match_reaction: Regex,
    tempo: Tempo<String>,
    delay_repeat: Duration,
}

impl<C, E> Trigger<C, E> {
    pub fn new(db: Rc<E>, client: Rc<C>, tempo: Tempo<String>, delay_repeat: Duration) -> Self {
        Self {
            db,
            client,
            tempo,
            delay_repeat,
            match_list: Regex::new("^!trigger list.*$").unwrap(),
            match_del: Regex::new("^!trigger del \"(.+)\".*").unwrap(),
            match_reaction: Regex::new("^!trigger reaction \"([^\"]+)\" [:\"]([^:]+)[:\"].*$")
                .unwrap(),
            match_text: Regex::new("^!trigger text \"([^\"]+)\" \"([^\"]+)\".*$").unwrap(),
        }
    }
}

impl<C, E> Handler for Trigger<C, E>
where
    C: client::Sender,
    E: db::Trigger,
{
    type Data = GenericPost;

    fn name(&self) -> &str {
        "trigger"
    }
    fn help(&self) -> Option<&str> {
        Some(
            "```
!trigger list
!trigger text \"trigger\" \"me\"\
!trigger reaction \"trigger\" :emoji:
!trigger del \"trigger\"
```",
        )
    }

    fn handle(&mut self, data: GenericPost) -> Result {
        let message: &str = &data.message;

        if !message.starts_with("!trigger ") {
            let res = self.db.search(&data.team_id)?;
            for t in res {
                let tb = &t.triggered_by;
                let tb_word = &format!(" {} ", tb);
                let tb_start = &format!("{} ", tb);
                let tb_end = &format!(" {}", tb);
                if message.contains(tb_word)
                    || message.starts_with(tb_start)
                    || message.ends_with(tb_end)
                    || message == t.triggered_by
                {
                    if t.text_.is_some() {
                        let tempo_key = format!("{}{}{}", &data.team_id, &data.channel_id, tb);

                        // sending this trigger has been delayed
                        if self.tempo.exists(tempo_key.clone()) {
                            self.tempo.set(tempo_key.clone(), self.delay_repeat);
                            break;
                        }

                        // now, delay this trigger
                        self.tempo.set(tempo_key.clone(), self.delay_repeat);
                        self.client.reply(data, &t.text_.unwrap())?;
                        break;
                    } else {
                        self.client.reaction(data.clone(), &t.emoji.unwrap())?;
                    }
                }
            }
            return Ok(());
        }

        if self.match_list.is_match(message) {
            let res = self.db.list(&data.team_id)?;
            return Ok(self.client.send_trigger_list(res, data)?);
        }

        match self.match_text.captures(message) {
            Some(captures) => {
                let _ = self.db.add_text(
                    &data.team_id,
                    captures.get(1).unwrap().as_str(),
                    captures.get(2).unwrap().as_str(),
                );
                return Ok(self.client.reaction(data, "ok_hand")?);
            }
            None => {}
        }

        match self.match_reaction.captures(message) {
            Some(captures) => {
                let _ = self.db.add_emoji(
                    &data.team_id,
                    captures.get(1).unwrap().as_str(),
                    captures.get(2).unwrap().as_str(),
                );
                return Ok(self.client.reaction(data, "ok_hand")?);
            }
            None => {}
        }
        match self.match_del.captures(message) {
            Some(captures) => {
                let _ = self
                    .db
                    .del(&data.team_id, captures.get(1).unwrap().as_str())?;
                return Ok(self.client.reaction(data, "ok_hand")?);
            }
            None => {}
        }

        Ok(())
    }
}

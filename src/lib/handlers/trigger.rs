use crate::client;
use crate::db;
use crate::db::tempo::Tempo;
use crate::handlers::{Handler, Result};
use crate::models::GenericPost;
use crate::models::Trigger as MTrigger;
use regex::escape as escape_re;
use regex::Regex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

pub fn compile_trigger(trigger: &str) -> std::result::Result<Regex, regex::Error> {
    let re = format!("(?ms)^.*({}).*$", escape_re(trigger));
    Regex::new(&re)
}

pub fn valid_match(re: &Regex, message: &str) -> bool {
    let captures = re.captures(message);

    if captures.is_none() {
        return false;
    }

    if let Some(capture) = captures.unwrap().get(1) {
        if capture.start() == 0 {
            if let Some(after) = message[capture.end()..].chars().next() {
                return after.is_whitespace();
            }
            return true;
        }

        if capture.end() == message.as_bytes().len() {
            if let Some(before) = message[capture.start() - 1..].chars().next() {
                return before.is_whitespace();
            }
            return true;
        }

        let mut chars = message.chars();
        let cs = chars.nth(capture.start() - 1).unwrap();
        let ce = chars.nth(capture.end() - capture.start()).unwrap();
        return cs.is_whitespace() && ce.is_whitespace();
    }

    false
}

pub struct Trigger<C, E> {
    db: Rc<E>,
    client: Rc<C>,
    match_list: Regex,
    match_del: Regex,
    match_text: Regex,
    match_reaction: Regex,
    tempo: Tempo<String>,
    delay_repeat: Duration,
    trig_cache: RefCell<HashMap<String, Regex>>,
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
            match_reaction: Regex::new("^!trigger reaction \"([^\"]+)\" [:\"]([^:]+)[:\"].*$").unwrap(),
            match_text: Regex::new("^!trigger text \"([^\"]+)\" \"([^\"]+)\".*$").unwrap(),
            trig_cache: RefCell::new(HashMap::new()),
        }
    }

    pub fn match_trigger(&self, message: &str, trigger: &String) -> bool {
        if !self.trig_cache.borrow().contains_key(trigger) {
            match compile_trigger(trigger) {
                Ok(re) => {
                    self.trig_cache.borrow_mut().insert(trigger.clone(), re);
                }
                Err(_) => {
                    // this case should never happen if compile_trigger was used before inserting.
                    // but old data or manual inserts can break this assumption.
                    return false;
                }
            };
        }

        // at this point the insertion was successful, so we can get a second time
        // an safely .unwrap(). TODO: use brain and do it more cleverly.
        let b = self.trig_cache.borrow();
        let re = b.get(trigger).unwrap();
        return valid_match(re, message);
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

    fn help(&self) -> Option<String> {
        Some(format!(
            "```
Automatically react to a given text in each received message on channels where the bot is present.

There is a per channel antispam of 3 seconds, avoiding a heated channel to be polluted by the bot.

A per [channel, trigger] antispam is effective and currently configured at {} seconds.

!trigger list
!trigger text \"trigger\" \"me\"
!trigger reaction \"trigger\" :emoji:
!trigger del \"trigger\"
```",
            self.delay_repeat.as_secs()
        ))
    }

    fn handle(&self, post: &GenericPost) -> Result {
        let message = post.message.as_str();

        if !message.starts_with("!trigger ") {
            // check or set a per channel rate limit to avoid spamming in heated discussions.
            let tempo_rate = format!("{}{}--global-channel-rate-limit", &post.team_id, &post.channel_id);
            if self.tempo.exists(&tempo_rate) {
                return Ok(());
            }
            self.tempo.set(tempo_rate.clone(), Duration::from_secs(3));

            // search for triggers in the message
            let team_triggers = self.db.search(&post.team_id)?;
            for t in team_triggers
                .iter()
                .filter(|tt| self.match_trigger(&post.message, &tt.triggered_by))
                .collect::<Vec<&MTrigger>>()
            {
                let tempo_key = format!(
                    "{}{}{}--trigger-channel-rate-limit",
                    &post.team_id, &post.channel_id, t.triggered_by
                );

                // sending this trigger has been delayed
                if self.tempo.exists(&tempo_key) {
                    continue;
                }
                self.tempo.set(tempo_key.clone(), self.delay_repeat);

                if t.text_.is_some() {
                    // text is sorted after emoji, so we can break here: emoji were already processed.
                    self.client.reply(post, t.text_.as_ref().unwrap())?;
                    break;
                } else {
                    // send all emoji reactions
                    self.client.reaction(post, &t.emoji.as_ref().unwrap())?;
                }
            }
            return Ok(());
        }

        if self.match_list.is_match(message) {
            let res = self.db.list(&post.team_id)?;
            return Ok(self.client.send_trigger_list(res, post)?);
        }

        match self.match_text.captures(message) {
            Some(captures) => {
                let trigger = captures.get(1).unwrap().as_str();

                // prevent insertion of broken triggers.
                if let Err(e) = compile_trigger(trigger) {
                    return Ok(self.client.reply(post, &e.to_string())?);
                }

                let _ = self.db.add_text(&post.team_id, trigger, captures.get(2).unwrap().as_str());
                return Ok(self.client.reaction(post, "ok_hand")?);
            }
            None => {}
        }

        match self.match_reaction.captures(message) {
            Some(captures) => {
                let trigger = captures.get(1).unwrap().as_str();

                // prevent insertion of broken triggers.
                if let Err(e) = compile_trigger(trigger) {
                    return Ok(self.client.reply(post, &e.to_string())?);
                }

                let _ = self.db.add_emoji(&post.team_id, trigger, captures.get(2).unwrap().as_str());
                return Ok(self.client.reaction(post, "ok_hand")?);
            }
            None => {}
        }

        match self.match_del.captures(message) {
            Some(captures) => {
                let _ = self.db.del(&post.team_id, captures.get(1).unwrap().as_str())?;
                return Ok(self.client.reaction(post, "ok_hand")?);
            }
            None => {}
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vm(message: &str) -> bool {
        let re = compile_trigger("trig").unwrap();
        valid_match(&re, message)
    }

    #[test]
    fn test_valid_match_left() {
        assert!(vm("trig "));
        assert!(vm("trig yes"));
        assert!(!vm("trigno"));
    }

    #[test]
    fn test_valid_match_end() {
        assert!(vm(" trig"));
        assert!(vm("ye trig"));
        assert!(!vm("notrig"));
    }

    #[test]
    fn test_valid_match_between() {
        assert!(vm("trig"));
        assert!(vm(" trig "));
        assert!(vm("yes trig yes"));
        assert!(!vm("notrigno"));
    }

    #[test]
    fn test_valid_match_nbsp() {
        // TODO: support nbsp
        assert!(!vm("nbsp\u{A0}trig\u{A0}nbsp"));
    }
}

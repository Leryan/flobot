use crate::client::Client;
use crate::db;
use crate::models::GenericPost;

use regex::Regex;

pub trait Handler<C> {
    type Data;
    fn handle(&self, data: Self::Data, client: &C);
}

pub struct Debug {
    name: String,
}

impl Debug {
    pub fn new(name: &str) -> Self {
        Debug {
            name: String::from(name),
        }
    }
}

impl<C: Client> Handler<C> for Debug {
    type Data = GenericPost;

    fn handle(&self, data: GenericPost, _client: &C) {
        println!("handler {:?} -> {:?}", self.name, data)
    }
}

pub struct Edit<C, E> {
    match_list: Regex,
    match_del: Regex,
    match_add: Regex,
    match_edit: Regex,
    phantom: std::marker::PhantomData<C>,
    db: E,
}

impl<C: Client, E: db::Edits> Edit<C, E> {
    pub fn new(db: E) -> Self {
        Self {
            match_list: Regex::new("^!edits list.*$").unwrap(),
            match_del: Regex::new("^!edits del \"(.+)\".*").unwrap(),
            match_add: Regex::new("^!edits add \"(.+)\" \"(.+)\".*").unwrap(),
            match_edit: Regex::new("^!e (.+)").unwrap(),
            phantom: std::marker::PhantomData,
            db: db,
        }
    }

    fn handle_edit(&self, post: GenericPost, client: &C, captured: &str) {
        let res = self
            .db
            .find(post.user_id.as_str(), post.team_id.as_str(), captured)
            .unwrap_or(None);

        match res {
            Some(edit) => {
                if edit.replace_with_text.is_some() {
                    client.edit_post_message(
                        post.id.as_str(),
                        edit.replace_with_text.as_ref().unwrap().as_str(),
                    )
                } else if edit.replace_with_file.is_some() {
                    client.unimplemented(post)
                }
            }
            _ => {}
        }
    }

    fn handle_del_team(&self, post: GenericPost, client: &C, captured: &str) {
        let res = self.db.del_team(post.team_id.as_str(), captured);

        match res {
            Ok(_) => client.send_reaction(post, "ok_hand"),
            Err(_) => {}
        }
    }

    fn handle_add(&self, post: GenericPost, client: &C, word: &str, replace: &str) {
        if word == replace {
            return client.send_reply(post.clone(), "aha, aha… il est boubourse :3");
        }

        if post.team_id.as_str() == "" {
            return client.send_reply(post.clone(), "je sais pas encore faire des edits privés :/");
        }

        match self.db.add_team(post.team_id.as_str(), word, replace) {
            Ok(_) => client.send_reaction(post, "ok_hand"),
            Err(e) => client.debug(format!("inserting edit: {:?}", e).as_str()),
        }
    }

    fn handle_list(&self, post: GenericPost, client: &C) {
        let res = self.db.list(post.team_id.as_str()).unwrap_or(vec![]); // FIXME: handle error

        if res.len() == 0 {
            return client.send_reply(post, "yen a pô :GE:");
        }

        let mut out = String::from("Remplacements disponibles:\n");
        for edit_ in res {
            out.push_str(
                format!(
                    " * `{}` -> {}",
                    edit_.edit,
                    edit_.replace_with_text.unwrap_or("".to_string())
                )
                .as_str(),
            );
        }

        client.send_reply(post, out.as_str())
    }

    fn handle_post(&self, post: GenericPost, client: &C) {
        let message = post.message.as_str();

        match self.match_edit.captures(message) {
            Some(captures) => {
                return self.handle_edit(post.clone(), client, captures.get(1).unwrap().as_str())
            }
            None => {}
        }

        match self.match_add.captures(message) {
            Some(captures) => {
                return self.handle_add(
                    post.clone(),
                    client,
                    captures.get(1).unwrap().as_str(),
                    captures.get(2).unwrap().as_str(),
                )
            }
            None => {}
        }

        match self.match_list.captures(message) {
            Some(_captures) => return self.handle_list(post, client),
            None => {}
        }

        match self.match_del.captures(message) {
            Some(captures) => {
                return self.handle_del_team(
                    post.clone(),
                    client,
                    captures.get(1).unwrap().as_str(),
                )
            }
            None => {}
        }
    }
}

impl<C: Client, E: db::Edits> Handler<C> for Edit<C, E> {
    type Data = GenericPost;

    fn handle(&self, data: GenericPost, client: &C) {
        self.handle_post(data, client)
    }
}

pub struct Trigger<E> {
    db: E,
    match_list: Regex,
    match_del: Regex,
    match_text: Regex,
    match_reaction: Regex,
}

impl<E: db::Trigger> Trigger<E> {
    pub fn new(db: E) -> Self {
        Self {
            db: db,
            match_list: Regex::new("^!trigger list.*$").unwrap(),
            match_del: Regex::new("^!trigger del \"(.+)\".*").unwrap(),
            match_reaction: Regex::new("^!trigger reaction \"([^\"]+)\" [:\"]([^:]+)[:\"].*$")
                .unwrap(),
            match_text: Regex::new("^!trigger text \"([^\"]+)\" \"([^\"]+)\".*$").unwrap(),
        }
    }
}

impl<C: Client, E: db::Trigger> Handler<C> for Trigger<E> {
    type Data = GenericPost;

    fn handle(&self, data: GenericPost, client: &C) {
        let message = data.message.as_str();

        if !message.starts_with("!trigger ") {
            let res = self.db.search(data.team_id.as_str()).unwrap_or(vec![]);
            for t in res {
                let tb = t.triggered_by.as_str();
                let tb_word = format!(" {} ", tb);
                let tb_start = format!("{} ", tb);
                let tb_end = format!(" {}", tb);
                if message.contains(tb_word.as_str())
                    || message.starts_with(tb_start.as_str())
                    || message.ends_with(tb_end.as_str())
                    || message == t.triggered_by.as_str()
                {
                    if t.text_.is_some() {
                        client.send_reply(data.clone(), t.text_.unwrap().as_str());
                        break;
                    } else {
                        client.send_reaction(data.clone(), t.emoji.unwrap().as_str());
                    }
                }
            }
            return;
        }

        if self.match_list.is_match(message) {
            let res = self.db.list(data.team_id.as_str());
            match res {
                Ok(it) => {
                    client.send_trigger_list(it, data);
                }
                Err(e) => {
                    let mut post = data;
                    post.message = format!("tout cassé: {:?}", e).to_string();
                    client.send_post(post);
                }
            }
            return;
        }
        match self.match_text.captures(message) {
            Some(captures) => {
                let _ = self.db.add_text(
                    data.team_id.as_str(),
                    captures.get(1).unwrap().as_str(),
                    captures.get(2).unwrap().as_str(),
                );
                return client.send_reaction(data.clone(), "ok_hand");
            }
            None => {}
        }

        match self.match_reaction.captures(message) {
            Some(captures) => {
                let _ = self.db.add_emoji(
                    data.team_id.as_str(),
                    captures.get(1).unwrap().as_str(),
                    captures.get(2).unwrap().as_str(),
                );
                return client.send_reaction(data.clone(), "ok_hand");
            }
            None => {}
        }
        match self.match_del.captures(message) {
            Some(captures) => {
                let _ = self
                    .db
                    .del(data.team_id.as_str(), captures.get(1).unwrap().as_str());
                return client.send_reaction(data.clone(), "ok_hand");
            }
            None => {}
        }
    }
}

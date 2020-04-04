use crate::client::Client;
use crate::models::db;
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

pub struct Edit<C> {
    db: diesel::SqliteConnection,
    match_list: Regex,
    match_del: Regex,
    match_add: Regex,
    match_edit: Regex,
    phantom: std::marker::PhantomData<C>,
}

impl<C: Client> Edit<C> {
    pub fn new(db: diesel::SqliteConnection) -> Self {
        Self {
            db: db,
            match_list: Regex::new("^!edits list.*$").unwrap(),
            match_del: Regex::new("^!edits del \"(.+)\".*").unwrap(),
            match_add: Regex::new("^!edits add \"(.+)\" \"(.+)\".*").unwrap(),
            match_edit: Regex::new("^!e (.+)").unwrap(),
            phantom: std::marker::PhantomData,
        }
    }

    fn handle_edit(&self, post: GenericPost, client: &C, captured: &str) {
        use crate::diesel::prelude::*;
        use crate::schema::edits::dsl::*;
        let res = edits
            .filter(
                team_id
                    .eq(post.team_id.as_str())
                    .or(user_id.eq(post.user_id.as_str()))
                    .and(edit.eq(captured.trim())),
            )
            .order_by(user_id) // user edits first, then team
            .limit(1)
            .load::<db::Edit>(&self.db)
            .unwrap_or(vec![]);

        if res.len() == 1 {
            let edit_ = &res[0];
            if edit_.replace_with_text.is_some() {
                client.edit_post_message(
                    post.id.as_str(),
                    edit_.replace_with_text.as_ref().unwrap().as_str(),
                )
            } else if edit_.replace_with_file.is_some() {
                client.unimplemented(post)
            }
        }
    }

    fn handle_del_team(&self, post: GenericPost, client: &C, captured: &str) {
        use crate::diesel::prelude::*;
        use crate::schema::edits::dsl::*;

        let filter = edits.filter(team_id.eq(post.team_id.as_str()).and(edit.eq(captured)));
        let res = diesel::delete(filter).execute(&self.db);

        match res {
            Ok(_) => client.send_reaction(post, "ok_hand"),
            Err(_) => {}
        }
    }

    fn handle_add(&self, post: GenericPost, client: &C, word: &str, replace: &str) {
        use crate::diesel::prelude::*;
        use crate::schema::edits::dsl::*;

        if word == replace {
            return client.send_reply(post.clone(), "aha, aha… il est boubourse :3");
        }

        if post.team_id.as_str() == "" {
            return client.send_reply(post.clone(), "je sais pas encore faire des edits privés :/");
        }

        let mut team_id_ = None;
        let mut user_id_ = None;

        match post.team_id.as_str() {
            "" => {
                user_id_ = Some(post.user_id.as_str());
            }
            e => {
                team_id_ = Some(e);
            }
        };

        let edit_ = db::NewEdit {
            edit: word,
            replace_with_text: Some(replace),
            replace_with_file: None,
            team_id: team_id_,
            user_id: user_id_,
        };

        match diesel::insert_into(edits).values(&edit_).execute(&self.db) {
            Ok(_) => client.send_reaction(post, "ok_hand"),
            Err(e) => client.debug(format!("inserting edit: {:?}", e).as_str()),
        }
    }

    fn handle_list(&self, post: GenericPost, client: &C) {
        use crate::diesel::prelude::*;
        use crate::schema::edits::dsl::*;
        let res = edits
            .filter(team_id.eq(post.team_id.as_str()))
            .order_by(user_id) // user edits first, then team
            .limit(1)
            .load::<db::Edit>(&self.db)
            .unwrap_or(vec![]);

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

impl<C: Client> Handler<C> for Edit<C> {
    type Data = GenericPost;

    fn handle(&self, data: GenericPost, client: &C) {
        self.handle_post(data, client)
    }
}

pub struct Trigger {
    dbpool: diesel::SqliteConnection,
    match_list: Regex,
    match_del: Regex,
    match_text: Regex,
    match_reaction: Regex,
}

impl Trigger {
    pub fn new(pool: diesel::SqliteConnection) -> Self {
        Self {
            dbpool: pool,
            match_list: Regex::new("^!trigger list.*$").unwrap(),
            match_del: Regex::new("^!trigger del \"(.+)\".*").unwrap(),
            match_reaction: Regex::new("^!trigger reaction \"([^\"]+)\" [:\"]([^:]+)[:\"].*$")
                .unwrap(),
            match_text: Regex::new("^!trigger text \"([^\"]+)\" \"([^\"]+)\".*$").unwrap(),
        }
    }
}

impl<C: Client> Handler<C> for Trigger {
    type Data = GenericPost;

    fn handle(&self, data: GenericPost, client: &C) {
        use crate::diesel::prelude::*;
        use crate::schema::trigger::dsl::*;
        let message = data.message.as_str();

        if !message.starts_with("!trigger ") {
            let res = trigger
                .filter(team_id.eq(data.team_id.as_str()))
                .order_by(text_) // emojis first -> all emoji triggers processed first, then text
                .load::<db::Trigger>(&self.dbpool)
                .unwrap_or(vec![]);
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
            let res = trigger
                .filter(team_id.eq(data.team_id.as_str()))
                .order(triggered_by.asc())
                .load::<db::Trigger>(&self.dbpool);
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
                use crate::schema::trigger;

                let new_trigger = db::NewTrigger {
                    triggered_by: captures.get(1).unwrap().as_str(),
                    emoji: None,
                    text_: Some(captures.get(2).unwrap().as_str()),
                    team_id: data.team_id.as_str(),
                };

                let _res = diesel::insert_into(trigger::table)
                    .values(&new_trigger)
                    .execute(&self.dbpool);

                return client.send_reaction(data.clone(), "ok_hand");
            }
            None => {}
        }

        match self.match_reaction.captures(message) {
            Some(captures) => {
                use crate::schema::trigger;

                let new_trigger = db::NewTrigger {
                    triggered_by: captures.get(1).unwrap().as_str(),
                    emoji: Some(captures.get(2).unwrap().as_str()),
                    text_: None,
                    team_id: data.team_id.as_str(),
                };

                let _res = diesel::insert_into(trigger::table)
                    .values(&new_trigger)
                    .execute(&self.dbpool);

                return client.send_reaction(data.clone(), "ok_hand");
            }
            None => {}
        }
        match self.match_del.captures(message) {
            Some(captures) => {
                let tb = captures.get(1).unwrap().as_str();
                let tid = data.team_id.as_str();
                let filter = triggered_by.eq(tb).and(team_id.eq(tid));
                let _res = diesel::delete(trigger.filter(filter)).execute(&self.dbpool);

                return client.send_reaction(data.clone(), "ok_hand");
            }
            None => {}
        }
    }
}

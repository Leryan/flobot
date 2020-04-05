use super::{Handler, Result};
use crate::client::Client;
use crate::db;
use crate::models::GenericPost;

use regex::Regex;

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

    fn handle_edit(&self, post: GenericPost, client: &C, captured: &str) -> Result {
        let res = self
            .db
            .find(post.user_id.as_str(), post.team_id.as_str(), captured)?;

        match res {
            Some(edit) => {
                if edit.replace_with_text.is_some() {
                    client.edit_post_message(
                        post.id.as_str(),
                        edit.replace_with_text.as_ref().unwrap().as_str(),
                    )?;
                } else if edit.replace_with_file.is_some() {
                    client.unimplemented(post)?;
                }
            }
            _ => {}
        };

        Ok(())
    }

    fn handle_del_team(&self, post: GenericPost, client: &C, captured: &str) -> Result {
        let _ = self.db.del_team(post.team_id.as_str(), captured)?;
        Ok(client.send_reaction(post, "ok_hand")?)
    }

    fn handle_add(&self, post: GenericPost, client: &C, word: &str, replace: &str) -> Result {
        if word == replace {
            return Ok(client.send_reply(post.clone(), "aha, aha… il est boubourse :3")?);
        }

        if post.team_id.as_str() == "" {
            return Ok(
                client.send_reply(post.clone(), "je sais pas encore faire des edits privés :/")?
            );
        }

        let _ = self.db.add_team(post.team_id.as_str(), word, replace)?;
        Ok(client.send_reaction(post, "ok_hand")?)
    }

    fn handle_list(&self, post: GenericPost, client: &C) -> Result {
        let res = self.db.list(post.team_id.as_str())?;

        if res.len() == 0 {
            return Ok(client.send_reply(post, "yen a pô :GE:")?);
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

        Ok(client.send_reply(post, out.as_str())?)
    }

    fn handle_post(&self, post: GenericPost, client: &C) -> Result {
        let message = post.message.as_str();

        match self.match_edit.captures(message) {
            Some(captures) => {
                return self.handle_edit(post.clone(), client, captures.get(1).unwrap().as_str());
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
                );
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
                );
            }
            None => {}
        }

        Ok(())
    }
}

impl<C: Client, E: db::Edits> Handler<C> for Edit<C, E> {
    type Data = GenericPost;

    fn handle(&self, data: GenericPost, client: &C) -> Result {
        self.handle_post(data, client)
    }
}

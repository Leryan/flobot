use crate::client::Client;
use crate::db;
use crate::handlers::{Handler, Result};
use crate::models::GenericPost;
use std::rc::Rc;

use regex::Regex;

pub struct Edit<C, E> {
    match_list: Regex,
    match_del: Regex,
    match_add: Regex,
    match_edit: Regex,
    phantom: std::marker::PhantomData<C>,
    db: Rc<E>,
}

impl<C: Client, E: db::Edits> Edit<C, E> {
    pub fn new(db: Rc<E>) -> Self {
        Self {
            match_list: Regex::new("^!edits list.*$").unwrap(),
            match_del: Regex::new("^!edits del \"(.+)\".*").unwrap(),
            match_add: Regex::new("^!edits add \"(.+)\" \"(.+)\".*").unwrap(),
            match_edit: Regex::new("^!e (.+)").unwrap(),
            phantom: std::marker::PhantomData,
            db,
        }
    }

    fn handle_edit(&self, post: GenericPost, client: &C, captured: &str) -> Result {
        let res = self.db.find(&post.user_id, &post.team_id, captured)?;

        match res {
            Some(edit) => {
                if edit.replace_with_text.is_some() {
                    client.edit_post_message(&post.id, &edit.replace_with_text.unwrap())?;
                } else if edit.replace_with_file.is_some() {
                    client.unimplemented(post)?;
                }
            }
            _ => {}
        };

        Ok(())
    }

    fn handle_del_team(&self, post: GenericPost, client: &C, captured: &str) -> Result {
        let _ = self.db.del_team(&post.team_id, captured)?;
        Ok(client.send_reaction(post, "ok_hand")?)
    }

    fn handle_add(&self, post: GenericPost, client: &C, word: &str, replace: &str) -> Result {
        if word == replace {
            return Ok(client.send_reply(post, "aha, aha… il est boubourse :3")?);
        }

        if post.team_id == "" {
            return Ok(client.send_reply(post, "je sais pas encore faire des edits privés :/")?);
        }

        let _ = self.db.add_team(&post.team_id, word, replace)?;
        Ok(client.send_reaction(post, "ok_hand")?)
    }

    fn handle_list(&self, post: GenericPost, client: &C) -> Result {
        let res = self.db.list(&post.team_id)?;

        if res.len() == 0 {
            return Ok(client.send_reply(post, "yen a pô :GE:")?);
        }

        let mut out = String::from("Remplacements disponibles:\n");
        for edit_ in res {
            out.push_str(&format!(
                " * `{}` -> {}",
                edit_.edit,
                edit_.replace_with_text.unwrap_or("".to_string())
            ));
        }

        Ok(client.send_reply(post, &out)?)
    }

    fn handle_post(&self, post: GenericPost, client: &C) -> Result {
        let message = post.message.clone();

        match self.match_edit.captures(&message) {
            Some(captures) => {
                return self.handle_edit(post, client, captures.get(1).unwrap().as_str());
            }
            None => {}
        };

        match self.match_add.captures(&message) {
            Some(captures) => {
                return self.handle_add(
                    post,
                    client,
                    captures.get(1).unwrap().as_str(),
                    captures.get(2).unwrap().as_str(),
                );
            }
            None => {}
        };

        match self.match_list.captures(&message) {
            Some(_captures) => return self.handle_list(post, client),
            None => {}
        };

        match self.match_del.captures(&message) {
            Some(captures) => {
                return self.handle_del_team(post, client, captures.get(1).unwrap().as_str());
            }
            None => {}
        };

        Ok(())
    }
}

impl<C: Client, E: db::Edits> Handler<C> for Edit<C, E> {
    type Data = GenericPost;

    fn name(&self) -> &str {
        "edits"
    }
    fn help(&self) -> Option<&str> {
        Some(
            "```
!edits list
!edits add \"edit\" \"replace\"
!edits del \"edit\"
!e edit
```",
        )
    }

    fn handle(&mut self, data: GenericPost, client: &C) -> Result {
        self.handle_post(data, client)
    }
}

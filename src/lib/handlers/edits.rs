use crate::db;
use flobot_lib::client;
use flobot_lib::handler::{Handler, Result};
use flobot_lib::models::Post;
use std::sync::Arc;

use regex::Regex;

pub struct Edit<C, E> {
    match_list: Regex,
    match_del: Regex,
    match_add: Regex,
    match_edit: Regex,
    client: C,
    db: Arc<E>,
}

impl<C, E> Edit<C, E>
where
    C: client::Sender,
    E: db::Edits,
{
    pub fn new(db: Arc<E>, client: C) -> Self {
        Self {
            match_list: Regex::new("^!edits list.*$").unwrap(),
            match_del: Regex::new("^!edits del \"(.+)\".*").unwrap(),
            match_add: Regex::new("^!edits add \"(.+)\" \"(.+)\".*").unwrap(),
            match_edit: Regex::new("^!e (.+)").unwrap(),
            db,
            client,
        }
    }

    fn handle_edit(&self, post: &Post, captured: &str) -> Result {
        let res = self.db.find(&post.user_id, &post.team_id, captured)?;

        match res {
            Some(edit) => {
                if edit.replace_with_text.is_some() {
                    self.client
                        .edit(&post.id, &edit.replace_with_text.unwrap())?;
                } else if edit.replace_with_file.is_some() {
                    // unimplemented
                }
            }
            _ => {}
        };

        Ok(())
    }

    fn handle_del_team(&self, post: &Post, captured: &str) -> Result {
        let _ = self.db.del_team(&post.team_id, captured)?;
        Ok(self.client.reaction(post, "ok_hand")?)
    }

    fn handle_add(&self, post: &Post, word: &str, replace: &str) -> Result {
        if word == replace {
            return Ok(self.client.reply(post, "aha, aha… il est boubourse :3")?);
        }

        if post.team_id == "" {
            return Ok(self
                .client
                .reply(post, "je sais pas encore faire des edits privés :/")?);
        }

        let _ = self.db.add_team(&post.team_id, word, replace)?;
        Ok(self.client.reaction(post, "ok_hand")?)
    }

    fn handle_list(&self, post: &Post) -> Result {
        let res = self.db.list(&post.team_id)?;

        if res.len() == 0 {
            return Ok(self.client.reply(post, "yen a pô :GE:")?);
        }

        let mut out = String::from("Remplacements disponibles:\n");
        for edit_ in res {
            out.push_str(&format!(
                " * `{}` -> {}\n",
                edit_.edit,
                edit_.replace_with_text.unwrap_or("".to_string())
            ));
        }

        Ok(self.client.reply(post, &out)?)
    }

    fn handle_post(&self, post: &Post) -> Result {
        let message = post.message.clone();

        match self.match_edit.captures(&message) {
            Some(captures) => {
                return self.handle_edit(post, captures.get(1).unwrap().as_str());
            }
            None => {}
        };

        match self.match_add.captures(&message) {
            Some(captures) => {
                return self.handle_add(
                    post,
                    captures.get(1).unwrap().as_str(),
                    captures.get(2).unwrap().as_str(),
                );
            }
            None => {}
        };

        match self.match_list.captures(&message) {
            Some(_captures) => return self.handle_list(post),
            None => {}
        };

        match self.match_del.captures(&message) {
            Some(captures) => {
                return self.handle_del_team(post, captures.get(1).unwrap().as_str());
            }
            None => {}
        };

        Ok(())
    }
}

impl<C, E> Handler for Edit<C, E>
where
    C: client::Sender,
    E: db::Edits,
{
    type Data = Post;

    fn name(&self) -> String {
        "edits".into()
    }
    fn help(&self) -> Option<String> {
        Some(
            "```
!edits list
!edits add \"edit\" \"replace\"
!edits del \"edit\"
!e edit
```"
            .to_string(),
        )
    }

    fn handle(&self, post: &Post) -> Result {
        self.handle_post(post)
    }
}

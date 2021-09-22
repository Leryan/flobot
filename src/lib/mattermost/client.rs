use super::models::*;
use crate::client::{Channel, Error, Getter, Notifier, Result, Sender};
use crate::conf::Conf;
use crate::models::*;
use uuid::Uuid;

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            return Error::Timeout(e.to_string());
        }

        if e.is_status() {
            return Error::Status(e.to_string());
        }

        if e.is_builder() {
            return Error::Body(e.to_string());
        }

        Error::Other(e.to_string())
    }
}

#[derive(Clone)]
pub struct Mattermost {
    pub cfg: Conf,
    me: Me,
    client: reqwest::blocking::Client,
}

impl Mattermost {
    pub fn new(cfg: Conf) -> Result<Self> {
        let client = reqwest::blocking::Client::new();
        let me: Me = client
            .get(&format!("{}/users/me", &cfg.api_url))
            .bearer_auth(&cfg.token)
            .send()?
            .json()?;
        println!("my user id: {}", me.id);
        Ok(Mattermost { cfg: cfg, me, client })
    }

    fn url(&self, add: &str) -> String {
        let mut url = self.cfg.api_url.clone();
        url.push_str(add);
        url
    }
}

impl Channel for Mattermost {
    fn create_private(&self, team_id: &str, name: &str, users: &Vec<String>) -> Result<String> {
        let mut enc_buf = Uuid::encode_buffer();
        let uuid = Uuid::new_v4().to_simple().encode_lower(&mut enc_buf);
        let channel_name = format!("{}-{}", name, uuid).to_lowercase().clone();
        let mmchannel = CreateChannel {
            team_id: team_id,
            name: channel_name.as_str(),
            display_name: name,
            type_: "P",
        };

        let r: GenericID = self
            .client
            .post(&self.url("/channels"))
            .bearer_auth(&self.cfg.token)
            .json(&mmchannel)
            .send()?
            .json()?;

        for user_id in users.iter() {
            let uid = UserID { user_id: user_id.clone() };
            self.client
                .post(&self.url(format!("/channels/{}/members", r.id).as_str()))
                .bearer_auth(&self.cfg.token)
                .json(&uid)
                .send()?;
        }

        Ok(r.id)
    }

    fn archive_channel(&self, channel_id: &str) -> Result<()> {
        self.client
            .delete(&self.url(format!("/channels/{}", channel_id).as_str()))
            .bearer_auth(&self.cfg.token)
            .send()?;

        Ok(())
    }
}

impl Sender for Mattermost {
    fn post(&self, post: &GenericPost) -> Result<()> {
        let mmpost = NewPost {
            channel_id: post.channel_id.clone(),
            create_at: 0,
            file_ids: vec![],
            message: &post.message,
            metadata: Metadata {},
            props: Props {},
            update_at: 0,
            user_id: self.me.id.clone(),
            parent_id: None,
            root_id: None,
        };
        self.client
            .post(&self.url("/posts"))
            .bearer_auth(&self.cfg.token)
            .json(&mmpost)
            .send()?;
        Ok(())
    }

    fn message(&self, post: &GenericPost, message: &str) -> Result<()> {
        let mut post = post.clone();
        post.message = message.to_string();
        self.post(&post)
    }

    fn reaction(&self, post: &GenericPost, reaction: &str) -> Result<()> {
        let reaction = Reaction {
            user_id: self.me.id.clone(),
            post_id: post.id.clone(),
            emoji_name: String::from(reaction),
        };
        self.client
            .post(&self.url("/reactions"))
            .bearer_auth(&self.cfg.token)
            .json(&reaction)
            .send()?;
        Ok(())
    }

    fn reply(&self, post: &GenericPost, message: &str) -> Result<()> {
        let mmpost = NewPost {
            channel_id: post.channel_id.clone(),
            create_at: 0,
            file_ids: vec![],
            message: message,
            metadata: Metadata {},
            props: Props {},
            update_at: 0,
            user_id: self.me.id.clone(),
            parent_id: Some(post.id.clone()),
            root_id: Some(post.id.clone()),
        };
        self.client
            .post(&self.url("/posts"))
            .bearer_auth(&self.cfg.token)
            .json(&mmpost)
            .send()?;
        Ok(())
    }

    fn edit(&self, post_id: &str, message: &str) -> Result<()> {
        let edit = PostEdit {
            message: Some(message),
            file_ids: None,
        };

        self.client
            .put(&self.url(&format!("/posts/{}/patch", post_id)))
            .bearer_auth(&self.cfg.token)
            .json(&edit)
            .send()?;
        Ok(())
    }

    fn send_trigger_list(&self, triggers: Vec<Trigger>, from: &GenericPost) -> Result<()> {
        let mut l = String::from(format!("Ya {:?} triggers.\n", triggers.len()));
        let mut count = 0;

        for trigger in triggers {
            count += 1;
            if trigger.emoji.is_some() {
                l.push_str(&format!(" * `{}`: :{}:\n", trigger.triggered_by, trigger.emoji.unwrap()));
            } else {
                l.push_str(&format!(" * `{}`: {}\n", trigger.triggered_by, trigger.text_.unwrap()));
            }

            if count == 20 {
                self.message(from, &l)?;
                count = 0;
                l = String::new();
            }
        }

        if count > 0 {
            self.message(from, &l)?;
        }

        Ok(())
    }
}

impl Notifier for Mattermost {
    fn startup(&self, message: &str) -> Result<()> {
        let datetime = chrono::offset::Local::now();
        let mut post = GenericPost::with_message(&format!(
            "# Startup {:?} (local time)\n## Build Hash\n * `{}`\n{}",
            datetime,
            crate::BUILD_GIT_HASH,
            message
        ));
        post.channel_id = self.cfg.debug_channel.clone();
        self.post(&post)
    }

    fn debug(&self, message: &str) -> Result<()> {
        let mut post = GenericPost::with_message(message);
        post.channel_id = self.cfg.debug_channel.clone();
        self.post(&post)
    }

    fn error(&self, message: &str) -> Result<()> {
        self.debug(message)
    }
}

impl Getter for Mattermost {
    fn my_user_id(&self) -> &str {
        &self.me.id
    }

    fn users_by_ids(&self, ids: Vec<&str>) -> Result<Vec<GenericUser>> {
        let r = self
            .client
            .post(self.url("/users/ids"))
            .bearer_auth(&self.cfg.token)
            .json(&ids)
            .send()?;

        let users: Vec<User> = r.json()?;

        let mut fusers: Vec<GenericUser> = vec![];
        for u in users.iter() {
            fusers.push((*u).clone().into());
        }

        Ok(fusers)
    }
}

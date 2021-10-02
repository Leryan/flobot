#[derive(Clone, Debug)]
pub enum Event {
    Hello(Hello),
    Post(Post),
    Status(Status),
    Unsupported(String),
    PostEdited(PostEdited),
    Shutdown,
}

#[derive(Clone, Debug)]
pub struct Hello {
    pub server_string: String,
}

#[derive(Clone, Debug)]
pub struct Post {
    pub channel_id: String,
    pub message: String,
    pub user_id: String,
    pub root_id: String,
    pub parent_id: String,
    pub id: String,
    pub team_id: String,
}

#[derive(Clone, Debug)]
pub struct PostEdited {
    pub channel_id: String,
    pub message: String,
    pub user_id: String,
    pub root_id: String,
    pub parent_id: String,
    pub id: String,
}

impl Post {
    pub fn new() -> Self {
        Self {
            channel_id: "".to_string(),
            message: "".to_string(),
            user_id: "".to_string(),
            root_id: "".to_string(),
            parent_id: "".to_string(),
            id: "".to_string(),
            team_id: "".to_string(),
        }
    }

    pub fn with_message(message: &str) -> Self {
        let mut s = Self::new();
        s.message = message.to_string();
        s
    }

    pub fn nmessage(&self, message: &str) -> Self {
        let mut s = self.clone();
        s.message = message.to_string();
        s
    }

    pub fn nchannel(&self, id: &str) -> Self {
        let mut s = self.clone();
        s.channel_id = id.to_string();
        s
    }
}

#[derive(Clone, Debug)]
pub enum StatusCode {
    OK,
    Error,
    Unknown,
    Unsupported,
}

#[derive(Clone, Debug)]
pub struct Status {
    pub code: StatusCode,
    pub error: Option<StatusError>,
}

#[derive(Clone, Debug)]
pub struct StatusError {
    pub message: String,
    pub detailed_error: String,
    pub request_id: Option<String>,
    pub status_code: i32,
}

pub struct User {
    pub id: String,
    pub username: String,
    pub display_name: String,
}

pub struct GenericMe {
    pub id: String,
}

impl StatusError {
    pub fn new_none() -> Self {
        Self {
            message: "none".to_string(),
            detailed_error: "".to_string(),
            request_id: None,
            status_code: 0,
        }
    }
}

// db
use diesel::Queryable;

#[derive(Debug, Queryable, Clone)]
pub struct Edit {
    pub id: i32,
    pub edit: String,
    pub team_id: Option<String>,
    pub user_id: Option<String>,
    pub replace_with_text: Option<String>,
    pub replace_with_file: Option<String>,
}

#[derive(Debug, Queryable)]
pub struct Trigger {
    pub id: i32,
    pub triggered_by: String,
    pub emoji: Option<String>,
    pub text_: Option<String>,
    pub team_id: String,
}

#[derive(Debug, Queryable)]
pub struct Blague {
    pub id: i32,
    pub team_id: String,
    pub text: String,
}

#[derive(Debug, Queryable)]
pub struct SMSContact {
    pub id: i32,
    pub team_id: String,
    pub name: String,
    pub number: String,
    pub last_sending_unixts: i64,
}

#[derive(Debug, Queryable)]
pub struct SMSPrepare {
    pub id: i32,
    pub team_id: String,
    pub contact_id: i32,
    pub trigname: String,
    pub name: String,
    pub text: String,
}

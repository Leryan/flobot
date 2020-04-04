pub mod db;
pub mod mattermost;

#[derive(Clone, Debug)]
pub enum GenericEvent {
    Hello(GenericHello),
    Post(GenericPost),
    Status(GenericStatus),
    Unsupported(String),
}

#[derive(Clone, Debug)]
pub struct GenericHello {
    pub server_string: String,
    pub my_user_id: String,
}

#[derive(Clone, Debug)]
pub struct GenericPost {
    pub channel_id: String,
    pub message: String,
    pub user_id: String,
    pub root_id: String,
    pub parent_id: String,
    pub id: String,
    pub team_id: String,
}

#[derive(Clone, Debug)]
pub enum StatusCode {
    OK,
    Error,
    Unknown,
    Unsupported,
}

#[derive(Clone, Debug)]
pub struct GenericStatus {
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

pub struct Me {
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

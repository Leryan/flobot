pub mod mattermost;

#[derive(Clone, Debug)]
pub enum Event {
    Post(Post),
    Status(Status),
    Unsupported(String),
}

#[derive(Clone, Debug)]
pub struct Post {
    pub channel_id: String,
    pub message: String,
    pub user_id: String,
    pub root_id: String,
    pub parent_id: String,
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

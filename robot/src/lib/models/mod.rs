pub mod mattermost;

pub enum Event {
    Post(Post),
    Unsupported
}

pub struct Post {
    channel_id: String,
    message: String,
    user_id: String,
    root_id: String,
    parent_id: String
}
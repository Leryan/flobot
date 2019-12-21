pub mod mattermost;

#[derive(Clone, Debug)]
pub enum Event {
    Post(Post),
}

#[derive(Clone, Debug)]
pub struct Post {
    pub channel_id: String,
    pub message: String,
    pub user_id: String,
    pub root_id: String,
    pub parent_id: String,
}

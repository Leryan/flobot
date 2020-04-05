use crate::models::*;
use crossbeam::crossbeam_channel::Sender;

#[derive(Debug)]
pub enum Error {
    Send(String),
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait EventClient {
    fn listen(&self, sender: Sender<GenericEvent>);
    fn client(&self) -> Box<dyn Client>;
}

pub trait Client {
    fn set_my_user_id(&mut self, user_id: &str) -> Result<()>;
    fn send_post(&self, post: GenericPost) -> Result<()>;
    fn send_reaction(&self, post: GenericPost, reaction: &str) -> Result<()>;
    fn send_reply(&self, post: GenericPost, message: &str) -> Result<()>;
    fn send_message(&self, from: GenericPost, message: &str) -> Result<()>;
    fn send_trigger_list(&self, triggers: Vec<Trigger>, from: GenericPost) -> Result<()>;
    fn edit_post_message(&self, post_id: &str, message: &str) -> Result<()>;
    fn notify_startup(&self) -> Result<()>;
    fn unimplemented(&self, post: GenericPost) -> Result<()>;
    fn debug(&self, message: &str) -> Result<()>;
}

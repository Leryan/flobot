use crate::models::db::Trigger;
use crate::models::*;
use crossbeam::crossbeam_channel::Sender;

pub mod mattermost;

pub trait EventClient {
    fn listen(&self, sender: Sender<GenericEvent>);
    fn client(&self) -> Box<dyn Client>;
}

pub trait Client {
    fn set_my_user_id(&mut self, user_id: &str);
    fn send_post(&self, post: GenericPost);
    fn send_reaction(&self, post: GenericPost, reaction: &str);
    fn send_reply(&self, post: GenericPost, message: &str);
    fn send_message(&self, from: GenericPost, message: &str);
    fn send_trigger_list(&self, triggers: Vec<Trigger>, from: GenericPost);
}

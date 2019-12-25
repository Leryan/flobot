use crate::models::Event;
use crossbeam::crossbeam_channel::Sender;

pub mod mattermost;

pub trait Client {
    fn listen(&self, sender: Sender<Event>);
}

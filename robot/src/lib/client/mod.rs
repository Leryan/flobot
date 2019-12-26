use crate::models::*;
use crossbeam::crossbeam_channel::Sender;

pub mod mattermost;

pub trait EventClient {
    fn listen(&self, sender: Sender<Event>);
    fn client(&self) -> Box<dyn Client>;
}

pub trait Client {
    fn me(&self) -> Me;
}

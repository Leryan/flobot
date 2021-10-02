//! Basic signal handling using the libc crate.
//! Once a signal has been registered it cannot be de-registered.
//! Empirically tested.
//!
//! See ctrl-c or signal-hook crates for cross platform libs.
//!
//! See example for a ready to use boilerplate.

extern crate libc;
use std::sync::{mpsc, Arc};

use libc as c;
use libc::signal as c_signal;

#[derive(Debug)]
pub enum Signal {
    SIGALRM,
    SIGINT,
    SIGTERM,
    SIGUSR1,
    SIGUSR2,
    OTHER(CSig),
}

/// ```
/// use libc;
/// use flobot::signal;
///
/// let sig: signal::CSig = libc::SIGUSR1;
/// ```
pub type CSig = i32;
type Recv = mpsc::Receiver<CSig>;
type Send = mpsc::Sender<CSig>;

static mut SIGNAL_RECEIVER: Option<Arc<Recv>> = None;
static mut SIGNAL_SENDER: Option<Send> = None;

unsafe fn ensure() -> (&'static Arc<Recv>, &'static Send) {
    if SIGNAL_RECEIVER.is_none() || SIGNAL_SENDER.is_none() {
        let (sender, receiver) = mpsc::channel();
        SIGNAL_RECEIVER = Some(Arc::new(receiver));
        SIGNAL_SENDER = Some(sender)
    }

    (
        SIGNAL_RECEIVER.as_ref().unwrap(),
        SIGNAL_SENDER.as_ref().unwrap(),
    )
}

fn signal_callback(sig: CSig) {
    unsafe {
        if let Err(e) = ensure().1.send(sig) {
            panic!("signal -> cannot send sig {:?}: {:?}", sig, e);
        }
    }
}

pub fn recv() -> Option<Signal> {
    unsafe {
        match ensure().0.recv().ok() {
            Some(c_sig) => match c_sig {
                c::SIGTERM => Some(Signal::SIGTERM),
                c::SIGINT => Some(Signal::SIGINT),
                c::SIGALRM => Some(Signal::SIGALRM),
                c::SIGUSR1 => Some(Signal::SIGUSR1),
                c::SIGUSR2 => Some(Signal::SIGUSR2),
                _ => Some(Signal::OTHER(c_sig)),
            },
            None => None,
        }
    }
}

unsafe fn unsafe_register(c_sig: CSig) {
    // https://users.rust-lang.org/t/function-pointers-and-raw-function-pointers/15152/7
    let ptr = (&(signal_callback as *const fn(CSig)) as *const *const fn(CSig))
        as *const fn(CSig); // wtf?
    c_signal(c_sig, *ptr as usize);
}

/// ```
/// use flobot::signal;
///
/// signal::register(signal::Signal::SIGINT);
/// // issue signal…
/// // use blocking call signal::recv() in a thread loop for example.
/// ```
pub fn register(sig: Signal) {
    unsafe {
        match sig {
            Signal::SIGINT => unsafe_register(c::SIGINT),
            Signal::SIGTERM => unsafe_register(c::SIGTERM),
            Signal::SIGALRM => unsafe_register(c::SIGALRM),
            Signal::SIGUSR1 => unsafe_register(c::SIGUSR1),
            Signal::SIGUSR2 => unsafe_register(c::SIGUSR2),
            Signal::OTHER(c_sig) => unsafe_register(c_sig),
        };
    };
}

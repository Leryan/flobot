use signal_libc::signal::{self, Signal};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

fn main() {
    let stop = Arc::new(AtomicBool::new(false));

    signal::register(Signal::SIGINT);
    signal::register(Signal::SIGTERM);
    // you can register a Signal::OTHER(CSig) but then handling might get more complicated.
    // if possible, reserve OTHER(_) for unhandled signals (signal::register not called).

    // work thread
    let stop_t = stop.clone();
    let t = thread::spawn(move || {
        while !stop_t.load(Ordering::Relaxed) {
            println!("do stuff…");
            thread::sleep(std::time::Duration::from_secs(1));
        }
    });

    // signal handling
    loop {
        if let Some(sig) = signal::recv() {
            match sig {
                Signal::OTHER(c_sig) => println!("received signal {}", c_sig),
                _ => {
                    stop.store(true, Ordering::Relaxed);
                    break;
                }
            }
        }
    }

    // patiently wait
    let res_t = t.join();
    println!("thread gracefully stopped: {:?}", res_t);
}

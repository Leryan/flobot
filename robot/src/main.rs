use crossbeam::crossbeam_channel::unbounded;
use crossbeam::sync::WaitGroup;
use robot::client::mattermost::Mattermost;
use robot::client::*;
use robot::conf::Conf;
use robot::handlers;
use robot::instance::Instance;
use robot::middleware;
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Conf::new()?;

    let (sender, receiver) = unbounded();
    let wg = WaitGroup::new();

    {
        let wg = wg.clone();
        let cfg = cfg.clone();
        thread::spawn(move || {
            println!("launch client thread");
            Mattermost::new(cfg).listen(sender);
            drop(wg);
        });
    }

    for i in 0..cfg.threads {
        let mrecv = receiver.clone();
        let cfg = cfg.clone();
        let wg = wg.clone();
        thread::spawn(move || {
            println!("launch instance thread {:?}/{:?}", i+1, cfg.threads);
            let client = Mattermost::new(cfg);
            Instance::new(&client)
                .add_middleware(Box::new(middleware::Debug::new("middleware 1")))
                .add_post_handler(Box::new(handlers::Debug::new("post handler")))
                .run(mrecv);
            drop(wg);
        });
    }

    wg.wait();

    Ok(())
}

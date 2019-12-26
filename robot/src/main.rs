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

    for _i in 0..4 {
        let mrecv = receiver.clone();
        let mcfg = cfg.clone();
        let wg = wg.clone();
        thread::spawn(move || {
            let client = Mattermost::new(mcfg);
            Instance::new(&client)
                .add_middleware(Box::new(middleware::Debug::new("middleware 1")))
                .add_post_handler(Box::new(handlers::Debug::new("post handler")))
                .run(mrecv);
            drop(wg);
        });
    }

    {
        let wg = wg.clone();
        thread::spawn(move || {
            Mattermost::new(cfg).listen(sender);
            drop(wg);
        });
    }

    wg.wait();

    Ok(())
}

use crossbeam::crossbeam_channel::unbounded;
use robot::client::mattermost::Mattermost;
use robot::client::*;
use robot::conf::Conf;
use robot::handlers;
use robot::instance::Instance;
use robot::middleware;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Conf::new()?;
    println!("{:?}", cfg);

    let (sender, receiver) = unbounded();

    let mut threads = vec![];
    for _i in 0..2 {
        let mrecv = receiver.clone();
        let mcfg = cfg.clone();
        let inst = std::thread::spawn(move || {
            let client = Mattermost::new(mcfg);
            Instance::new(&client)
                .add_middleware(Box::new(middleware::Debug::new("middleware 1")))
                .add_post_handler(Box::new(handlers::Debug::new("post handler")))
                .run(mrecv);
        });
        threads.push(inst);
    }

    let listener = std::thread::spawn(move || {
        Mattermost::new(cfg).listen(sender);
    });

    threads.push(listener);

    for i in threads {
        let _ = i.join();
    }

    Ok(())
}

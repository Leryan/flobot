use crossbeam::crossbeam_channel::unbounded;
use robot::client::mattermost::Mattermost;
use robot::client::Client;
use robot::conf::Conf;
use robot::handlers;
use robot::instance::Instance;
use robot::middleware;

fn instance_factory(instance: &mut Instance) -> &mut Instance {
    instance
        .add_middleware(Box::new(middleware::Debug::new("middleware 1")))
        .add_post_handler(Box::new(handlers::Debug::new("post handler")))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Conf::new()?;
    println!("{:?}", cfg);

    let (sender, receiver) = unbounded();
    let mut threads = vec![];

    for _i in 0..4 {
        let mrecv = receiver.clone();
        let inst = std::thread::spawn(move || {
            instance_factory(&mut Instance::new()).run(mrecv);
        });
        threads.push(inst);
    }

    let mattermost = Mattermost::new(cfg);
    mattermost.listen(sender);

    for i in threads {
        let _ = i.join();
    }

    Ok(())
}

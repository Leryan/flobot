use robot::conf::Conf;
use robot::handlers;
use robot::instance::Instance;
use robot::middleware;
use robot::models::{Event, Post};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Conf::new()?;
    println!("{:?}", cfg);

    let mut instance = Instance::new();
    let instance = instance
        .add_middleware(Box::new(middleware::Debug {}))
        .add_middleware(Box::new(middleware::Debug {}))
        .add_post_handler(Box::new(handlers::Debug {}));

    instance.process(Event::Post(Post {
        channel_id: "some_id".to_string(),
        message: "hello world".to_string(),
        parent_id: "".to_string(),
        root_id: "".to_string(),
        user_id: "".to_string(),
    }));

    Ok(())
}

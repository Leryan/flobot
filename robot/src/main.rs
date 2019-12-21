use robot::conf::Conf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Conf::new()?;
    println!("{:?}", cfg);

    Ok(())
}

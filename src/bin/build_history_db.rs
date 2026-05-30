use shogun::game::build_history_database;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    build_history_database("assets/data/history.sqlite")?;
    println!("generated assets/data/history.sqlite");
    Ok(())
}

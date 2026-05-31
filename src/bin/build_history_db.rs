use shogun::game::{SqliteHistoricalCatalog, build_history_database};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(SqliteHistoricalCatalog::default_path);
    build_history_database(&path)?;
    println!("generated {}", path.display());
    Ok(())
}

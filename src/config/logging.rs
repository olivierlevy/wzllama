use anyhow::Result;
use crate::config::paths;

pub fn init() -> Result<()> {
    let log_file = paths::log_dir().join("wzllama.log");
    let file = std::fs::File::create(&log_file)?;
    env_logger::Builder::new()
        .target(env_logger::Target::Pipe(Box::new(file)))
        .filter_level(log::LevelFilter::Debug)
        .init();
    Ok(())
}
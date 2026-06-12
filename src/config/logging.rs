use crate::config::paths;
use anyhow::Result;

pub fn init() -> Result<()> {
    let log_file = paths::log_dir().join("wzllama.log");
    let file = std::fs::File::create(&log_file)?;
    env_logger::Builder::new()
        .target(env_logger::Target::Pipe(Box::new(file)))
        .filter_level(log::LevelFilter::Debug)
        .init();
    Ok(())
}

/// Copy embedded i18n files to user directory on first run
pub fn install_embedded_i18n() -> Result<()> {
    let i18n_dir = paths::i18n_dir();

    // Check if i18n files already exist
    if i18n_dir.join("fr.json").exists() {
        return Ok(());
    }

    std::fs::create_dir_all(&i18n_dir)?;

    // Try to find config/i18n relative to executable or current directory
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));

    let candidates: Vec<std::path::PathBuf> = vec![
        // Relative to current working directory
        std::path::PathBuf::from("config/i18n"),
        // Relative to executable directory (target/release/ or target/debug/)
        exe_dir
            .as_ref()
            .map(|p| p.join("../../config/i18n"))
            .unwrap_or_default(),
    ];

    for embedded_path in candidates {
        if embedded_path.exists() {
            for entry in std::fs::read_dir(embedded_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "json") {
                    let content = std::fs::read_to_string(&path)?;
                    std::fs::write(i18n_dir.join(path.file_name().unwrap()), content)?;
                }
            }
            return Ok(());
        }
    }

    Ok(())
}

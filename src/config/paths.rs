use std::path::PathBuf;

pub fn home() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

pub fn wzllama_dir() -> PathBuf {
    home().join(".wzllama")
}

pub fn config_dir() -> PathBuf {
    wzllama_dir().join("config")
}

pub fn i18n_dir() -> PathBuf {
    wzllama_dir().join("i18n")
}

pub fn log_dir() -> PathBuf {
    wzllama_dir().join("logs")
}

pub fn state_file() -> PathBuf {
    wzllama_dir().join("state.json")
}

pub fn ensure_dirs() -> anyhow::Result<()> {
    std::fs::create_dir_all(config_dir())?;
    std::fs::create_dir_all(i18n_dir())?;
    std::fs::create_dir_all(log_dir())?;
    Ok(())
}

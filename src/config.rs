use std::path::PathBuf;

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".config")))
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("garmin")
}

pub fn tokens_path() -> PathBuf {
    config_dir().join("tokens.json")
}

pub fn consumer_path() -> PathBuf {
    config_dir().join("consumer.json")
}

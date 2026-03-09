use std::path::PathBuf;

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("garmin")
}

pub fn tokens_path() -> PathBuf {
    config_dir().join("tokens.json")
}

use std::env;
use std::path::PathBuf;

fn home_dir() -> PathBuf {
    env::home_dir().expect("Home dir not found")
}

pub fn data_dir() -> PathBuf {
    let data_dir = std::env::var("XDG_DATA_HOME")
        .map_or_else(|_| home_dir().join(".local").join("share"), PathBuf::from);

    data_dir.join("simple-rss")
}

pub fn config_path() -> PathBuf {
    let config_dir =
        std::env::var("XDG_CONFIG_HOME").map_or_else(|_| home_dir().join(".config"), PathBuf::from);

    config_dir.join("simple-rss")
}

use std::{
    fs,
    io::{self, BufRead},
    path::Path,
};

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::path::{config_path, data_dir};

mod loader;

pub use loader::DataLoader;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub channel_name: String,
    pub title: String,
    pub description: Option<String>,
    pub pub_date: Option<DateTime<FixedOffset>>,
    pub link: String,

    pub read: bool,
}

#[derive(Default)]
pub struct Data {
    pub channels: Vec<String>,
    pub items: Vec<Item>,
}

impl Data {
    fn load() -> anyhow::Result<Self> {
        let items = load_items()?;
        let channels = load_channels()?;

        Ok(Self { items, channels })
    }

    fn save(&self) -> anyhow::Result<()> {
        let path = data_dir().join("data.json");
        create_root(&path)?;

        let file = fs::File::create(&path)?;
        let writer = io::BufWriter::new(file);
        serde_json::to_writer(writer, &self.items)?;
        Ok(())
    }
}

/// Creates all the directories that are needed to have a file at path.
///
/// Example:
/// `/foo/bar/baz.txt`: makes sure that path `/foo/bar` exists
fn create_root(path: impl AsRef<Path>) -> io::Result<()> {
    let exists = path.as_ref().parent().map(|p| p.exists());
    if let Some(false) = exists {
        fs::create_dir_all(&path)?;
    }

    Ok(())
}

fn open_file_read(path: impl AsRef<Path>) -> io::Result<fs::File> {
    fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)
}

fn load_items() -> io::Result<Vec<Item>> {
    let path = data_dir().join("data.json");
    create_root(&path)?;

    let file = open_file_read(&path)?;
    let reader = io::BufReader::new(file);
    let items = serde_json::from_reader(reader).unwrap_or_default();

    Ok(items)
}

fn load_channels() -> io::Result<Vec<String>> {
    let path = config_path();
    create_root(&path)?;

    let file = open_file_read(&path)?;
    let reader = io::BufReader::new(file);
    let channels: Result<Vec<_>, _> = reader.lines().collect();
    let channels = channels?
        .into_iter()
        .filter_map(|l| {
            let trimmed = l.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .collect();
    Ok(channels)
}

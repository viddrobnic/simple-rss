use std::{fs, io, path::Path};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub name: Option<String>,
    pub url: String,
}

#[derive(Default)]
pub struct Data {
    pub channels: Vec<Channel>,
    pub items: Vec<Item>,
    pub version: u16,
}

impl Data {
    pub fn load() -> anyhow::Result<Self> {
        let items = load_items()?;
        let channels = load_channels()?;

        Ok(Self {
            items,
            channels,
            version: 0,
        })
    }

    pub fn save(&self) -> anyhow::Result<()> {
        save_items(&self.items)?;
        save_channels(&self.channels)?;
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

fn save_items(items: &[Item]) -> io::Result<()> {
    let path = data_dir().join("data.json");
    create_root(&path)?;

    let file = fs::File::create(&path)?;
    let writer = io::BufWriter::new(file);
    serde_json::to_writer(writer, items)?;
    Ok(())
}

fn load_channels() -> io::Result<Vec<Channel>> {
    let path = config_path();
    create_root(&path)?;

    let file = open_file_read(&path)?;
    let reader = io::BufReader::new(file);
    let channels = serde_json::from_reader(reader).unwrap_or_default();
    Ok(channels)
}

fn save_channels(channels: &[Channel]) -> io::Result<()> {
    let path = config_path();
    create_root(&path)?;

    let file = fs::File::create(&path)?;
    let writer = io::BufWriter::new(file);
    serde_json::to_writer(writer, channels)?;
    Ok(())
}

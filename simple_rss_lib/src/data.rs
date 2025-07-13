use std::sync::MutexGuard;

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

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
}

pub enum RefreshStatus {
    Ok,
    Error,
}

pub trait Loader {
    /// Get data and return read lock.
    /// Warning: This lock shouldn't be used across await.
    fn get_data(&self) -> MutexGuard<Data>;

    /// Version of the data. Used by items to know when data is changed
    /// and re-render is needed. It is the loader's implementation responsibility
    /// to increase the version each time the data is changed.
    fn get_version(&self) -> u16;

    fn refresh(&mut self) -> impl Future<Output = RefreshStatus> + Send;

    /// Set item at given index to read.
    fn set_read(&mut self, index: usize, read: bool);

    fn load_item(url: &str) -> impl Future<Output = String> + Send;
}

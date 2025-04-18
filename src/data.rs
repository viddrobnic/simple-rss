#[allow(deprecated)]
use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::{fs, io, path::Path};

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{
    event::{Event, EventSender},
    path::{config_path, data_dir},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub link: Option<String>,

    pub read: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub title: String,
    pub description: String,
}

#[derive(Clone)]
pub struct DataLoader {
    sender: EventSender,

    data: Arc<RwLock<Data>>,
}

#[derive(Default)]
pub struct Data {
    pub channels: Vec<Channel>,
    pub items: Vec<Item>,
}

impl DataLoader {
    pub fn new(sender: EventSender) -> anyhow::Result<Self> {
        let data = Data::load()?;

        Ok(Self {
            sender,
            data: Arc::new(RwLock::new(data)),
        })
    }

    pub fn get_data(&self) -> RwLockReadGuard<Data> {
        self.data.read().unwrap()
    }

    pub fn save(&self) {}

    pub async fn load_item(&self, url: &str) {
        let resp = reqwest::get(url).await;
        let text = match resp {
            Err(err) => {
                format!("Failed loading item: {}", err)
            }
            Ok(resp) => match resp.text().await {
                Ok(data) => data,
                Err(err) => format!("Failed loading item: {}", err),
            },
        };

        let _ = self.sender.send(Event::LoadedItem(text));
    }

    pub async fn refresh(&mut self) {
        // TODO: Get data

        let mut lock = self.data.write().unwrap();
        lock.items = vec![];
        lock.channels = vec![]
    }
}

impl Data {
    fn load() -> anyhow::Result<Self> {
        let items = load(data_dir().join("data.json"))?;
        let channels = load(config_path())?;

        Ok(Self { items, channels })
    }
}

fn load<T: DeserializeOwned>(path: impl AsRef<Path>) -> io::Result<Vec<T>> {
    let exists = path.as_ref().parent().map(|p| p.exists());
    if let Some(false) = exists {
        fs::create_dir_all(&path)?;
    }

    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)?;

    let reader = io::BufReader::new(file);
    let items: Vec<T> = serde_json::from_reader(reader).unwrap_or_default();
    Ok(items)
}

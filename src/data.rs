use std::sync::{Arc, Mutex};

use crate::event::{Event, EventSender};

#[derive(Debug, Clone)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub link: Option<String>,

    pub read: bool,
}

#[derive(Debug, Clone)]
pub struct Channel {
    pub title: String,
    pub description: String,
}

#[derive(Clone)]
pub struct DataLoader {
    sender: EventSender,

    data: Arc<Mutex<Data>>,
}

#[derive(Default)]
struct Data {
    channels: Vec<Channel>,
    items: Vec<Item>,
}

impl DataLoader {
    pub fn new(sender: EventSender) -> Self {
        let data = Data::load();

        Self {
            sender,
            data: Arc::new(Mutex::new(data)),
        }
    }

    pub fn get_items(&self) -> Vec<Item> {
        let lock = self.data.lock().unwrap();
        lock.items.clone()
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

        let mut lock = self.data.lock().unwrap();
        lock.items = vec![];
        lock.channels = vec![]
    }
}

impl Data {
    fn load() -> Self {
        Self::default()
    }
}

use tokio::sync::mpsc;

use crate::event::{Event, EventHandler};

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

#[derive(Debug, Clone)]
pub struct Data {
    pub channels: Vec<Channel>,
    pub items: Vec<Item>,
}

#[derive(Clone)]
pub struct DataLoader {
    sender: mpsc::UnboundedSender<Event>,
}

impl DataLoader {
    pub fn new(sender: mpsc::UnboundedSender<Event>) -> Self {
        Self { sender }
    }

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
}

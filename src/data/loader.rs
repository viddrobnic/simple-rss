use std::sync::{Arc, RwLock, RwLockReadGuard};

use futures::future::join_all;

use crate::event::{Event, EventSender};

use super::{Data, Item};

#[derive(Clone)]
pub struct DataLoader {
    sender: EventSender,

    data: Arc<RwLock<Data>>,
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

        self.sender.send(Event::LoadedItem(text));
    }

    pub async fn refresh(&mut self) {
        // This syntax is used as workaround for clippy - making sure that lock is dropped before
        // await
        let channels = {
            let lock = self.data.read().unwrap();
            lock.channels.clone()
        };

        let res = join_all(channels.iter().map(|ch| get_channel(ch))).await;

        let mut items = vec![];
        let mut errors = vec![];
        for result in res {
            match result {
                Ok(mut itms) => items.append(&mut itms),
                Err(err) => errors.push(err),
            }
        }

        if !errors.is_empty() {
            // TODO: Report errors to data sender
        } else {
            let mut lock = self.data.write().unwrap();
            // TODO: Sort items
            lock.items = items;
        }
    }
}

async fn get_channel(url: &str) -> anyhow::Result<Vec<Item>> {
    // TODO: Get items
    Ok(vec![Item {
        id: "test".to_string(),
        title: "title".to_string(),
        description: Some("description".to_string()),
        link: Some("https://viddrobnic.com".to_string()),
        read: false,
    }])
}

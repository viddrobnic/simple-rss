use std::sync::{Arc, RwLock, RwLockReadGuard};

use chrono::DateTime;
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
            items.sort_by(|a, b| b.pub_date.cmp(&a.pub_date));
            lock.items = items;
        }
    }
}

async fn get_channel(url: &str) -> anyhow::Result<Vec<Item>> {
    let content = reqwest::get(url).await?.bytes().await?;
    let channel = rss::Channel::read_from(&content[..])?;

    let items: Vec<_> = channel
        .items
        .into_iter()
        .filter_map(|it| {
            Some(Item {
                id: format!("{}:{}", url, it.guid.map(|g| g.value)?),
                title: it.title?,
                description: it.description,
                pub_date: it
                    .pub_date
                    .and_then(|d| DateTime::parse_from_rfc2822(&d).ok()),
                link: it.link?,
                read: false,
            })
        })
        .collect();

    Ok(items)
}

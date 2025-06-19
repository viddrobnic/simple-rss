use std::{
    collections::HashSet,
    sync::{Arc, RwLock, RwLockReadGuard},
};

use chrono::FixedOffset;
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

    pub fn save(&self) -> anyhow::Result<()> {
        let lock = self.data.read().unwrap();
        lock.save()
    }

    /// Set item at given index to read.
    pub fn set_read(&mut self, index: usize, read: bool) {
        let mut lock = self.data.write().unwrap();
        lock.items[index].read = read;
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

        self.sender.send(Event::LoadedItem(text));
    }

    pub async fn refresh(&mut self) {
        self.sender
            .send(Event::ToastLoading("Refreshing".to_string()));

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

        if errors.is_empty() {
            items.sort_by(|a, b| b.pub_date.cmp(&a.pub_date));

            let mut lock = self.data.write().unwrap();
            let mut read_items = HashSet::new();
            for it in &lock.items {
                if it.read {
                    read_items.insert(it.id.clone());
                }
            }

            for it in items.iter_mut() {
                it.read = read_items.contains(&it.id);
            }

            lock.items = items;

            self.sender.send(Event::ToastHide);
        } else {
            self.sender
                .send(Event::ToastError("Failed to refresh data!".to_string()));
        }
    }
}

async fn get_channel(url: &str) -> anyhow::Result<Vec<Item>> {
    let content = reqwest::get(url).await?.bytes().await?;
    let feed = feed_rs::parser::parse(&content[..])?;

    let items: Vec<_> = feed
        .entries
        .into_iter()
        .filter_map(|it| {
            Some(Item {
                id: format!("{}:{}", url, it.id),
                channel_name: feed
                    .title
                    .as_ref()
                    .map_or("Unnamed Channel".to_string(), |t| t.content.clone()),
                title: it.title?.content,
                description: it.summary.map(|d| d.content),
                pub_date: it
                    .updated
                    .or(it.published)
                    .map(|p| p.with_timezone(&FixedOffset::east_opt(0).unwrap())),
                link: it.links.first()?.href.clone(),
                read: false,
            })
        })
        .collect();

    Ok(items)
}

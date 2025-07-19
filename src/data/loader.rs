use std::{
    collections::HashSet,
    ops::Deref,
    sync::{self, Arc, Mutex},
};

use chrono::FixedOffset;
use futures::future::join_all;
use simple_rss_lib::data::{Loader, RefreshStatus};

use super::{Channel, Data, Item, load_data};

pub struct LockGuard<'a>(sync::MutexGuard<'a, Data>);

impl<'a> Deref for LockGuard<'a> {
    type Target = Vec<Item>;

    fn deref(&self) -> &Self::Target {
        &self.0.items
    }
}

#[derive(Clone)]
pub struct DataLoader {
    version: Arc<Mutex<u16>>,
    data: Arc<Mutex<Data>>,
}

impl DataLoader {
    pub fn get_data(&self) -> sync::MutexGuard<Data> {
        self.data.lock().unwrap()
    }
}

impl Loader for DataLoader {
    type Guard<'a> = LockGuard<'a>;

    fn get_items(&self) -> Self::Guard<'_> {
        LockGuard(self.data.lock().unwrap())
    }

    fn get_version(&self) -> u16 {
        *self.version.lock().unwrap()
    }

    /// Set item at given index to read.
    fn set_read(&mut self, index: usize, read: bool) {
        let mut lock = self.data.lock().unwrap();
        lock.items[index].read = read;

        let mut version = self.version.lock().unwrap();
        *version += 1;
    }

    async fn load_item(url: &str) -> String {
        let resp = reqwest::get(url).await;
        match resp {
            Err(err) => {
                format!("Failed loading item: {err}")
            }
            Ok(resp) => match resp.text().await {
                Ok(data) => data,
                Err(err) => format!("Failed loading item: {err}"),
            },
        }
    }

    async fn refresh(&mut self) -> RefreshStatus {
        // This syntax is used as workaround for clippy - making sure that lock is dropped before
        // await
        let channels = {
            let lock = self.data.lock().unwrap();
            lock.channels.clone()
        };

        let res = join_all(channels.iter().map(get_channel)).await;

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

            let mut lock = self.data.lock().unwrap();
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

            let mut version = self.version.lock().unwrap();
            *version += 1;

            RefreshStatus::Ok
        } else {
            RefreshStatus::Error
        }
    }
}

impl DataLoader {
    pub fn new() -> anyhow::Result<Self> {
        let data = load_data()?;

        Ok(Self {
            data: Arc::new(Mutex::new(data)),
            version: Arc::new(Mutex::new(0)),
        })
    }
}

async fn get_channel(channel: &Channel) -> anyhow::Result<Vec<Item>> {
    let content = reqwest::get(&channel.url).await?.bytes().await?;
    let feed = feed_rs::parser::parse(&content[..])?;

    let items: Vec<_> = feed
        .entries
        .into_iter()
        .filter_map(|it| {
            Some(Item {
                id: format!("{}:{}", channel.url, it.id),
                channel_name: channel.name.as_ref().map_or_else(
                    || {
                        feed.title
                            .as_ref()
                            .map_or("Unnamed Channel".to_string(), |t| t.content.clone())
                    },
                    |v| v.clone(),
                ),
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

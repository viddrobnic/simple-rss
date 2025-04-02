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

use ratatui::{
    style::{Color, Stylize},
    text::{Line, Text},
    widgets::{Block, BorderType, List, ListItem, Widget},
};

use crate::data::Item;

pub struct ItemList<'a> {
    data: &'a [Item],
}

impl<'a> ItemList<'a> {
    pub fn new(data: &'a [Item]) -> Self {
        Self { data }
    }
}

impl Widget for ItemList<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(Line::from("Channels"));

        let list = List::new(
            self.data
                .iter()
                .map(|it| item_to_list_item(it, area.width as usize)),
        )
        .block(block);
        list.render(area, buf);
    }
}

fn item_to_list_item(it: &Item, width: usize) -> ListItem {
    let opts = textwrap::Options::new(width)
        .initial_indent("[ ] ")
        .subsequent_indent("    ")
        .break_words(true);

    let mut text = Text::default();

    let title = textwrap::wrap(&it.title, &opts);
    text.extend(
        title
            .iter()
            .map(|s| Line::from(s.clone()).bold().fg(Color::Green)),
    );

    let Some(desc) = &it.description else {
        text.push_line("");
        return ListItem::from(text);
    };

    let opts = textwrap::Options::new(width)
        .initial_indent("    ")
        .subsequent_indent("    ")
        .break_words(true);
    let desc = textwrap::wrap(desc, &opts);
    text.extend(desc);

    text.push_line("");
    ListItem::from(text)
}

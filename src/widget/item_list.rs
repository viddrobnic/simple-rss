use ratatui::{
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{
        Block, BorderType, List, ListItem, ListState, Scrollbar, ScrollbarOrientation,
        ScrollbarState, StatefulWidget, Widget,
    },
};

use crate::data::Item;

pub struct ItemList<'a> {
    data: &'a [Item],
    list_state: &'a mut ListState,
}

impl<'a> ItemList<'a> {
    pub fn new(data: &'a [Item], list_state: &'a mut ListState) -> Self {
        Self { data, list_state }
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

        // List
        let list = List::new(self.data.iter().enumerate().map(|(idx, it)| {
            item_to_list_item(
                it,
                self.list_state.selected() == Some(idx),
                area.width as usize,
            )
        }))
        .block(block)
        .highlight_style(Style::default().bg(Color::Blue));

        StatefulWidget::render(list, area, buf, self.list_state);

        // Scrollbar
        let scroll_bar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        let mut bar_state =
            ScrollbarState::new(self.data.len()).position(self.list_state.selected().unwrap_or(0));
        StatefulWidget::render(scroll_bar, area, buf, &mut bar_state);
    }
}

fn item_to_list_item(it: &Item, selected: bool, width: usize) -> ListItem {
    let opts = textwrap::Options::new(width)
        .initial_indent("[ ] ")
        .subsequent_indent("    ")
        .break_words(true);

    let mut text = Text::default();

    let title = textwrap::wrap(&it.title, &opts);
    let title_col = if selected {
        Color::LightGreen
    } else {
        Color::Green
    };
    text.extend(
        title
            .iter()
            .map(|s| Line::from(s.clone()).bold().fg(title_col)),
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

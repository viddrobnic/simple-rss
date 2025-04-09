use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{
        Block, BorderType, List, ListItem, ListState, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
};

use crate::{
    data::Item,
    event::{Event, EventState},
};

pub struct ItemList {
    focused: bool,

    data: Vec<Item>,
    list_state: ListState,
}

impl ItemList {
    pub fn new(data: Vec<Item>, focused: bool) -> Self {
        Self {
            focused,
            data,
            list_state: ListState::default(),
        }
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn handle_event(&mut self, event: &Event) -> EventState {
        EventState::NotConsumed
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let mut block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(Line::from("Channels"));
        if !self.focused {
            block = block.border_style(Color::Gray)
        }

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

        frame.render_stateful_widget(list, area, &mut self.list_state);

        // Scrollbar
        let scroll_bar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        let mut bar_state =
            ScrollbarState::new(self.data.len()).position(self.list_state.selected().unwrap_or(0));
        frame.render_stateful_widget(scroll_bar, area, &mut bar_state);
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

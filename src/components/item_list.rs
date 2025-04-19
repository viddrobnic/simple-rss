use crossterm::event::KeyCode;
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
    data::{DataLoader, Item},
    event::{Event, EventSender, EventState},
};

pub struct ItemList {
    focused: bool,

    list_state: ListState,

    event_tx: EventSender,
    data_loader: DataLoader,
}

impl ItemList {
    pub fn new(focused: bool, event_tx: EventSender, data_loader: DataLoader) -> Self {
        Self {
            focused,
            list_state: ListState::default(),
            event_tx,
            data_loader,
        }
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn handle_event(&mut self, event: &Event) -> EventState {
        if !self.focused {
            return EventState::NotConsumed;
        }

        match event {
            Event::Keyboard(key_event) => self.handle_keyboard_event(key_event.code),
            _ => EventState::NotConsumed,
        }
    }

    fn handle_keyboard_event(&mut self, key: KeyCode) -> EventState {
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                self.list_state.select_previous();
                EventState::Consumed
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.list_state.select_next();
                EventState::Consumed
            }
            KeyCode::Enter => {
                if let Some(selected) = self.list_state.selected() {
                    let data = self.data_loader.get_data();
                    if let Some(url) = &data.items[selected].link {
                        let url = url.clone();
                        let loader = self.data_loader.clone();
                        tokio::spawn(async move {
                            loader.load_item(&url).await;
                        });

                        self.event_tx.send(Event::StartLoadingItem);
                    }
                }

                EventState::Consumed
            }
            _ => EventState::NotConsumed,
        }
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let mut block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(Line::from("Items"));
        if !self.focused {
            block = block.border_style(Color::Gray)
        }

        // List
        let data = self.data_loader.get_data();
        let list = List::new(data.items.iter().enumerate().map(|(idx, it)| {
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
            ScrollbarState::new(data.items.len()).position(self.list_state.selected().unwrap_or(0));
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

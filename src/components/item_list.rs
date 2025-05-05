use crossterm::event::KeyCode;
use html2text::config;
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
        match event {
            Event::Keyboard(key_event) => self.handle_keyboard_event(key_event.code),
            _ => EventState::NotConsumed,
        }
    }

    fn handle_keyboard_event(&mut self, key: KeyCode) -> EventState {
        //  Handle open browser separately, because it's independent of focus.
        if key == KeyCode::Char('o') {
            if let Some(selected) = self.list_state.selected() {
                let data = self.data_loader.get_data();

                let url = &data.items[selected].link;
                let _ = webbrowser::open(url);

                // Set to read
                drop(data); // Drop lock to avoid race condition
                self.data_loader.set_read(selected, true);
            }

            return EventState::Consumed;
        }

        if !self.focused {
            return EventState::NotConsumed;
        }

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

                    // Start loading item
                    let url = data.items[selected].link.clone();
                    let loader = self.data_loader.clone();
                    tokio::spawn(async move {
                        loader.load_item(&url).await;
                    });
                    self.event_tx.send(Event::StartLoadingItem);

                    // Set to read
                    drop(data); // Drop lock to avoid race condition
                    self.data_loader.set_read(selected, true);
                }

                EventState::Consumed
            }
            KeyCode::Char(' ') => {
                if let Some(selected) = self.list_state.selected() {
                    let data = self.data_loader.get_data();
                    let new_read = !data.items[selected].read;

                    drop(data); // Drop to avoid race condition
                    self.data_loader.set_read(selected, new_read);
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
    // Title
    let mut opts = textwrap::Options::new(width - 2)
        .subsequent_indent("    ")
        .break_words(true);
    if it.read {
        opts = opts.initial_indent("[X] ")
    } else {
        opts = opts.initial_indent("[ ] ")
    }

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

    // Channel name
    let opts = textwrap::Options::new(width - 2)
        .initial_indent("    ")
        .subsequent_indent("    ")
        .break_words(true);
    let channel = textwrap::wrap(&it.channel_name, &opts);
    text.extend(
        channel
            .iter()
            .map(|s| Line::from(s.clone()).bold().fg(Color::Gray)),
    );

    // Description
    let Some(desc) = &it.description else {
        text.push_line("");
        return ListItem::from(text);
    };

    text.push_line("");

    // TODO: Optimize this, at least not run on every render
    let desc = config::plain_no_decorate()
        // width - 4 (space prefix) - 2 (buffer) = width - 6
        .string_from_read(desc.as_bytes(), width - 6)
        .unwrap_or_else(|_| desc.clone())
        .lines()
        .map(|line| format!("    {line}"))
        .collect::<Vec<_>>()
        .join("\n");

    text.extend(Text::from(desc));

    text.push_line("");
    ListItem::from(text)
}

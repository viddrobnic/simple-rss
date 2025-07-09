use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
};
use unicode_width::UnicodeWidthStr;

use crate::{
    data::{DataLoader, Item},
    event::{Event, EventSender, EventState},
};

pub struct ItemList {
    focused: bool,

    list_state: ListState,

    event_tx: EventSender,
    data_loader: DataLoader,

    render_cache: Option<RenderCache>,
}

struct RenderCache {
    list: List<'static>,
    width: u16,
    version: u16,
}

impl ItemList {
    pub fn new(focused: bool, event_tx: EventSender, data_loader: DataLoader) -> Self {
        Self {
            focused,
            list_state: ListState::default(),
            event_tx,
            data_loader,
            render_cache: None,
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
        let instructions = Line::from(vec![
            "Exit ".into(),
            "<Esc> / <q>  ".blue().bold(),
            "Help ".into(),
            "<?>".blue().bold(),
        ]);
        let mut block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(Line::from("Items"))
            .title_bottom(instructions.centered());
        if !self.focused {
            block = block.border_style(Color::Gray)
        }
        let list_area = block.inner(area);
        frame.render_widget(block, area);

        // List
        let mut list_state = self.list_state.clone();
        let list = self.get_render_cache(list_area);
        let nr_items = list.list.len();

        if nr_items == 0 {
            self.draw_empty(frame, list_area);
            return;
        }

        frame.render_stateful_widget(&list.list, list_area, &mut list_state);
        self.list_state = list_state;

        // Scrollbar
        let scroll_bar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        let mut bar_state =
            ScrollbarState::new(nr_items).position(self.list_state.selected().unwrap_or(0));
        frame.render_stateful_widget(scroll_bar, area, &mut bar_state);
    }

    fn draw_empty(&self, frame: &mut Frame, mut area: Rect) {
        let paragraph = Paragraph::new(vec![
            Line::from("Add channels to get started").bold(),
            Line::from(vec!["See ".into(), "simple-rss help".fg(Color::DarkGray)]),
        ])
        .centered();

        area.y = area.height / 2;
        frame.render_widget(paragraph, area);
    }

    fn recalculate_render_cache(&mut self, area: Rect) -> &RenderCache {
        let data = self.data_loader.get_data();
        let list = List::new(
            data.items
                .iter()
                .map(|it| item_to_list_item(it, area.width as usize)),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

        self.render_cache = Some(RenderCache {
            list,
            width: area.width,
            version: data.version,
        });

        self.render_cache.as_ref().unwrap()
    }

    fn get_render_cache(&mut self, area: Rect) -> &RenderCache {
        let Some(render_cache) = &self.render_cache else {
            return self.recalculate_render_cache(area);
        };

        let version = {
            let data = self.data_loader.get_data();
            data.version
        };

        if render_cache.width != area.width || render_cache.version != version {
            return self.recalculate_render_cache(area);
        }

        self.render_cache.as_ref().unwrap()
    }
}

fn item_to_list_item(it: &Item, width: usize) -> ListItem<'static> {
    // Title
    let mut opts = textwrap::Options::new(width - 1)
        .subsequent_indent("    ")
        .break_words(true);
    if it.read {
        opts = opts.initial_indent("[X] ")
    } else {
        opts = opts.initial_indent("[ ] ")
    }

    let mut text = Text::default();

    let title = textwrap::wrap(&it.title, &opts);
    text.extend(
        title
            .iter()
            .map(|s| Line::from(s.to_string()).bold().fg(Color::LightGreen)),
    );

    let opts = textwrap::Options::new(width - 2)
        .initial_indent("    ")
        .subsequent_indent("    ")
        .break_words(true);

    // Channel name
    let Some(date) = &it.pub_date else {
        let channel = textwrap::wrap(&it.channel_name, &opts);
        text.extend(
            channel
                .iter()
                .map(|s| Line::from(s.to_string()).bold().fg(Color::Gray)),
        );

        text.push_line("");
        return ListItem::from(text);
    };

    let pub_time = format!("{}", date.format("%Y-%m-%d"));

    // 4 spaces at the beginning
    let total_width = it.channel_name.width() + pub_time.width() + 4;

    // Everything can fit on one line, we can do the nice formatting.
    if total_width < width - 3 {
        // 3 = Some buffer to have space around things
        let mut line = Line::from("    ");
        line.push_span(Span::from(it.channel_name.clone()).bold().fg(Color::Gray));

        let space = width - total_width - 1;
        for _ in 0..space {
            line.push_span(" ");
        }

        line.push_span(Span::from(pub_time).fg(Color::Gray));

        text.push_line(line);
        text.push_line("");

        return ListItem::from(text);
    }

    // We have to split by lines
    let channel = textwrap::wrap(&it.channel_name, &opts);
    text.extend(
        channel
            .iter()
            .map(|s| Line::from(s.to_string()).bold().fg(Color::Gray)),
    );
    text.push_line(Line::from(format!("    {pub_time}")).fg(Color::Gray));

    text.push_line("");
    ListItem::from(text)
}

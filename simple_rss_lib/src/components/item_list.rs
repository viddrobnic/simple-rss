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
    data::{Item, Loader},
    event::{Event, EventSender, EventState, KeyboardEvent},
};

pub struct Config {
    pub custom_empty_list_msg: Option<Paragraph<'static>>,
    pub disable_read_status: bool,
    pub disable_channel_names: bool,
    pub disable_browser_open: bool,
}

pub struct ItemList<L: Loader> {
    config: Config,

    focused: bool,

    list_state: ListState,

    event_tx: EventSender,
    data_loader: L,

    render_cache: Option<RenderCache>,

    empty_list_message: Paragraph<'static>,
}

struct RenderCache {
    list: List<'static>,
    width: u16,
    version: u16,
}

impl<L: Loader> ItemList<L> {
    pub fn new(focused: bool, event_tx: EventSender, data_loader: L, config: Config) -> Self {
        let empty_list_message = config.custom_empty_list_msg.clone().unwrap_or_else(|| {
            Paragraph::new(vec![
                Line::from("Add channels to get started").bold(),
                Line::from(vec!["See ".into(), "simple-rss help".fg(Color::DarkGray)]),
            ])
            .centered()
        });

        Self {
            config,
            focused,
            list_state: ListState::default(),
            event_tx,
            data_loader,
            render_cache: None,
            empty_list_message,
        }
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn handle_event(&mut self, event: &Event) -> EventState {
        match event {
            Event::Keyboard(key_event) => self.handle_keyboard_event(*key_event),
            _ => EventState::Ignored,
        }
    }

    fn handle_keyboard_event(&mut self, event: KeyboardEvent) -> EventState {
        //  Handle open browser separately, because it's independent of focus.
        if event == KeyboardEvent::Open && !self.config.disable_browser_open {
            if let Some(selected) = self.list_state.selected() {
                let data = self.data_loader.get_data();

                let url = &data.items[selected].link;
                let _ = webbrowser::open(url);

                // Set to read
                if !self.config.disable_read_status {
                    drop(data); // Drop lock to avoid race condition
                    self.data_loader.set_read(selected, true);
                }
            }

            return EventState::Handled;
        }

        if !self.focused {
            return EventState::Ignored;
        }

        match event {
            KeyboardEvent::Up => {
                self.list_state.select_previous();
                EventState::Handled
            }
            KeyboardEvent::Down => {
                self.list_state.select_next();
                EventState::Handled
            }
            KeyboardEvent::Enter => {
                if let Some(selected) = self.list_state.selected() {
                    let data = self.data_loader.get_data();

                    // Start loading item
                    let url = data.items[selected].link.clone();
                    let sender = self.event_tx.clone();
                    tokio::spawn(async move {
                        let text = L::load_item(&url).await;
                        sender.send(Event::LoadedItem(text));
                    });

                    self.event_tx.send(Event::StartLoadingItem);

                    // Set to read
                    if !self.config.disable_read_status {
                        drop(data); // Drop lock to avoid race condition
                        self.data_loader.set_read(selected, true);
                    }
                }

                EventState::Handled
            }
            KeyboardEvent::Space => {
                if let Some(selected) = self.list_state.selected() {
                    let data = self.data_loader.get_data();
                    let new_read = !data.items[selected].read;

                    if !self.config.disable_read_status {
                        drop(data); // Drop to avoid race condition
                        self.data_loader.set_read(selected, new_read);
                    }
                }

                EventState::Handled
            }
            _ => EventState::Ignored,
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
        area.y = area.height / 2;
        frame.render_widget(&self.empty_list_message, area);
    }

    fn recalculate_render_cache(&mut self, area: Rect) -> &RenderCache {
        let data = self.data_loader.get_data();
        let list = List::new(
            data.items
                .iter()
                .map(|it| item_to_list_item(it, area.width as usize, &self.config)),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

        self.render_cache = Some(RenderCache {
            list,
            width: area.width,
            version: self.data_loader.get_version(),
        });

        self.render_cache.as_ref().unwrap()
    }

    fn get_render_cache(&mut self, area: Rect) -> &RenderCache {
        let Some(render_cache) = &self.render_cache else {
            return self.recalculate_render_cache(area);
        };

        let version = self.data_loader.get_version();

        if render_cache.width != area.width || render_cache.version != version {
            return self.recalculate_render_cache(area);
        }

        self.render_cache.as_ref().unwrap()
    }
}

fn item_to_list_item(it: &Item, width: usize, config: &Config) -> ListItem<'static> {
    // Title
    let mut opts = textwrap::Options::new(width - 1).break_words(true);
    if !config.disable_read_status {
        opts = opts.subsequent_indent("    ");

        if it.read {
            opts = opts.initial_indent("[X] ")
        } else {
            opts = opts.initial_indent("[ ] ")
        }
    }

    let mut text = Text::default();

    let title = textwrap::wrap(&it.title, &opts);
    text.extend(
        title
            .iter()
            .map(|s| Line::from(s.to_string()).bold().fg(Color::LightGreen)),
    );

    let mut opts = textwrap::Options::new(width - 2).break_words(true);
    if !config.disable_read_status {
        opts = opts.initial_indent("    ").subsequent_indent("    ");
    }

    // Channel name
    let Some(date) = &it.pub_date else {
        if !config.disable_channel_names {
            let channel = textwrap::wrap(&it.channel_name, &opts);
            text.extend(
                channel
                    .iter()
                    .map(|s| Line::from(s.to_string()).bold().fg(Color::Gray)),
            );
        }

        text.push_line("");
        return ListItem::from(text);
    };

    let pub_time = format!("{}", date.format("%Y-%m-%d"));

    if config.disable_channel_names {
        let line = if config.disable_read_status {
            Line::from(pub_time)
        } else {
            Line::from(format!("    {pub_time}"))
        };
        text.push_line(line.fg(Color::Gray).bold());

        text.push_line("");
        return ListItem::from(text);
    }

    // 4 spaces at the beginning
    let mut total_width = it.channel_name.width() + pub_time.width();
    if !config.disable_read_status {
        total_width += 4;
    }

    // Everything can fit on one line, we can do the nice formatting.
    if total_width < width - 3 {
        // 3 = Some buffer to have space around things
        let mut line = if config.disable_read_status {
            Line::default()
        } else {
            Line::from("    ")
        };

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

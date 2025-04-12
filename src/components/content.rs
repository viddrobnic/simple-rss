use crossterm::event::KeyCode;
use html2text::from_read;
use ratatui::{
    Frame,
    layout::Rect,
    style::Color,
    widgets::{Block, BorderType, Paragraph, Wrap},
};

use crate::event::{Event, EventState};

#[derive(Default)]
enum ContentState {
    #[default]
    Empty,
    Loading(u8),
    Data(String),
}

pub struct Content {
    focused: bool,
    state: ContentState,
}

impl Content {
    pub fn new(focused: bool) -> Self {
        Self {
            focused,
            state: ContentState::default(),
        }
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn handle_event(&mut self, event: &Event) -> EventState {
        match event {
            Event::Keyboard(key_event) => self.handle_keyboard_event(key_event.code),
            Event::Tick => match self.state {
                ContentState::Loading(tick) => {
                    self.state = ContentState::Loading(tick.wrapping_add(1));
                    EventState::NotConsumed
                }
                _ => EventState::NotConsumed,
            },
            Event::StartLoadingItem => {
                self.state = ContentState::Loading(0);

                // Do not consume this event, so that the parent can transition
                // the focused state.
                EventState::NotConsumed
            }
            Event::LoadedItem(text) => {
                self.state = ContentState::Data(text.clone());
                EventState::Consumed
            }
        }
    }

    fn handle_keyboard_event(&mut self, key: KeyCode) -> EventState {
        if !self.focused {
            return EventState::NotConsumed;
        }

        // TODO: Implement this
        EventState::NotConsumed
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        match self.state {
            ContentState::Empty => self.draw_empty(frame, area),
            ContentState::Loading(tick) => self.draw_loading(tick, frame, area),
            ContentState::Data(ref text) => self.draw_data(frame, area, text),
        }
    }

    fn draw_empty(&self, frame: &mut Frame, mut area: Rect) {
        let block = basic_block(self.focused);
        frame.render_widget(block, area);

        let paragraph = Paragraph::new("Select an item to get started").centered();

        area.y = area.height / 2;
        frame.render_widget(paragraph, area);
    }

    fn draw_loading(&self, tick: u8, frame: &mut Frame, mut area: Rect) {
        let block = basic_block(self.focused);
        frame.render_widget(block, area);

        let paragraph = Paragraph::new(format!("Loading {tick}")).centered();

        area.y = area.height / 2;
        frame.render_widget(paragraph, area);
    }

    fn draw_data(&self, frame: &mut Frame, area: Rect, text: &str) {
        let block = basic_block(self.focused);

        let text =
            from_read(text.as_bytes(), area.width as usize).unwrap_or_else(|_| text.to_string());

        let paragraph = Paragraph::new(text)
            .block(block)
            .wrap(Wrap { trim: true })
            .scroll((0, 0));

        frame.render_widget(paragraph, area);
    }
}

fn basic_block(selected: bool) -> Block<'static> {
    let mut block = Block::bordered().border_type(BorderType::Rounded);
    if !selected {
        block = block.border_style(Color::Gray);
    }

    block
}

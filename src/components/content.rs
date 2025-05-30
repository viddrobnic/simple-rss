use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Stylize},
    text::Line,
    widgets::{Block, BorderType, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

use crate::{
    event::{Event, EventState},
    html_render::render,
};

const SPINNER_FRAMES: [u32; 10] = [
    0x280B, // ⠋
    0x2819, // ⠙
    0x2839, // ⠹
    0x2838, // ⠸
    0x283C, // ⠼
    0x2834, // ⠴
    0x2826, // ⠦
    0x2827, // ⠧
    0x2807, // ⠇
    0x280F, // ⠏
];

#[derive(Default)]
enum ContentState {
    #[default]
    Empty,
    Loading(u8),
    Data(ContentStateData),
}

struct ContentStateData {
    raw_text: String,
    scroll_offset: usize,

    render_cache: Option<RenderCache>,
}

struct RenderCache {
    lines: Vec<Line<'static>>,
    render_width: u16,
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
                self.state = ContentState::Data(ContentStateData {
                    raw_text: text.clone(),
                    scroll_offset: 0,
                    render_cache: None,
                });

                EventState::Consumed
            }
        }
    }

    fn handle_keyboard_event(&mut self, key: KeyCode) -> EventState {
        if !self.focused {
            return EventState::NotConsumed;
        }

        match &mut self.state {
            ContentState::Data(data) => data.handle_keyboard_event(key),
            _ => EventState::NotConsumed,
        }
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        match self.state {
            ContentState::Empty => self.draw_empty(frame, area),
            ContentState::Loading(tick) => self.draw_loading(tick, frame, area),
            ContentState::Data(ref mut data) => data.draw(frame, area, self.focused),
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

        let ch = SPINNER_FRAMES[(tick as usize / 3) % SPINNER_FRAMES.len()];

        // Safe because chars are hardcoded
        let ch = unsafe { char::from_u32_unchecked(ch) };

        let paragraph = Paragraph::new(format!("Loading {ch}")).centered();

        area.y = area.height / 2;
        frame.render_widget(paragraph, area);
    }
}

fn basic_block(selected: bool) -> Block<'static> {
    let instructions = Line::from(vec![
        "Back ".into(),
        "<Esc> / <q> / <Left>  ".blue().bold(),
        "Focus ".into(),
        "<Right>".blue().bold(),
    ]);
    let mut block = Block::bordered()
        .border_type(BorderType::Rounded)
        .title_bottom(instructions.centered());
    if !selected {
        block = block.border_style(Color::Gray);
    }

    block
}

impl ContentStateData {
    fn handle_keyboard_event(&mut self, key: KeyCode) -> EventState {
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);

                EventState::Consumed
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let nr_lines = self.render_cache.as_ref().map(|c| c.lines.len());
                if let Some(nr_lines) = nr_lines {
                    self.scroll_offset += 1;
                    self.scroll_offset = self.scroll_offset.min(nr_lines - 1);
                }

                EventState::Consumed
            }
            _ => EventState::NotConsumed,
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect, focused: bool) {
        let scroll_offset = self.scroll_offset;
        let cache = self.get_render_cache(area);

        let block = basic_block(focused);

        let paragraph = Paragraph::new(cache.lines.clone())
            .block(block)
            .scroll((scroll_offset as u16, 0));

        frame.render_widget(paragraph, area);

        // Scrollbar
        let scroll_bar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        let mut bar_state = ScrollbarState::new(cache.lines.len()).position(scroll_offset);
        frame.render_stateful_widget(scroll_bar, area, &mut bar_state);
    }

    fn get_render_cache(&mut self, area: Rect) -> &RenderCache {
        let Some(render_cache) = &self.render_cache else {
            return self.recalculate_render_cache(area);
        };

        if render_cache.render_width != area.width {
            return self.recalculate_render_cache(area);
        }

        self.render_cache.as_ref().unwrap()
    }

    fn recalculate_render_cache(&mut self, area: Rect) -> &RenderCache {
        let lines = render(&self.raw_text, area.width as usize - 2, true);

        self.render_cache = Some(RenderCache {
            lines,
            render_width: area.width,
        });

        self.render_cache.as_ref().unwrap()
    }
}

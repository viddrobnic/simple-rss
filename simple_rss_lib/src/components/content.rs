use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Stylize},
    text::Line,
    widgets::{Block, BorderType, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

use crate::{
    event::{Event, EventState, KeyboardEvent},
    html_render::render,
};

use super::spinner_frame;

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
            Event::Keyboard(key_event) => self.handle_keyboard_event(*key_event),
            Event::Tick => match self.state {
                ContentState::Loading(tick) => {
                    self.state = ContentState::Loading(tick.wrapping_add(1));
                    EventState::Handled
                }
                _ => EventState::Ignored,
            },
            Event::StartLoadingItem => {
                self.state = ContentState::Loading(0);
                EventState::Handled
            }
            Event::LoadedItem(text) => {
                self.state = ContentState::Data(ContentStateData {
                    raw_text: text.clone(),
                    scroll_offset: 0,
                    render_cache: None,
                });

                EventState::Handled
            }
            Event::Toast(_) => EventState::Ignored,
        }
    }

    fn handle_keyboard_event(&mut self, event: KeyboardEvent) -> EventState {
        if !self.focused {
            return EventState::Ignored;
        }

        match &mut self.state {
            ContentState::Data(data) => data.handle_keyboard_event(event),
            _ => EventState::Ignored,
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

        let paragraph = Paragraph::new("Select an item to get started")
            .bold()
            .centered();

        area.y = area.height / 2;
        frame.render_widget(paragraph, area);
    }

    fn draw_loading(&self, tick: u8, frame: &mut Frame, mut area: Rect) {
        let block = basic_block(self.focused);
        frame.render_widget(block, area);

        let ch = spinner_frame(tick as usize);
        let paragraph = Paragraph::new(format!("Loading {ch}")).centered();

        area.y = area.height / 2;
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

impl ContentStateData {
    fn handle_keyboard_event(&mut self, key: KeyboardEvent) -> EventState {
        match key {
            KeyboardEvent::Up => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);

                EventState::Handled
            }
            KeyboardEvent::Down => {
                let nr_lines = self.render_cache.as_ref().map(|c| c.lines.len());
                if let Some(nr_lines) = nr_lines {
                    self.scroll_offset += 1;
                    self.scroll_offset = self.scroll_offset.min(nr_lines.saturating_sub(5));
                }

                EventState::Handled
            }
            _ => EventState::Ignored,
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect, focused: bool) {
        let scroll_offset = self.scroll_offset;
        let cache = self.get_render_cache(area);

        let block = basic_block(focused);
        frame.render_widget(block, area);

        let lines = cache
            .lines
            .iter()
            .skip(scroll_offset + 1)
            .take((area.height as usize) - 2);
        for (idx, line) in lines.enumerate() {
            frame.render_widget(
                line,
                Rect::new(area.x + 1, area.y + idx as u16 + 1, area.width - 2, 1),
            );
        }

        // Scrollbar
        let scroll_bar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        let mut bar_state =
            ScrollbarState::new(cache.lines.len().saturating_sub(5)).position(scroll_offset);
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

use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Clear, Paragraph},
};

use crate::event::{Event, EventState, TICK_FPS};

use super::spinner_frame;

#[derive(Default)]
pub enum Toast {
    #[default]
    Hidden,
    Loading {
        message: String,
        ticks: u32,
    },
    Error {
        error: String,
        ticks: u32,
    },
}

impl Toast {
    pub fn new() -> Self {
        Toast::default()
    }

    pub fn handle_event(&mut self, event: &Event) -> EventState {
        match event {
            Event::ToastLoading(msg) => {
                *self = Toast::Loading {
                    message: msg.to_string(),
                    ticks: 0,
                };
                EventState::Consumed
            }
            Event::ToastError(msg) => {
                *self = Toast::Error {
                    error: msg.to_string(),
                    ticks: 0,
                };
                EventState::Consumed
            }
            Event::ToastHide => {
                *self = Toast::Hidden;
                EventState::Consumed
            }
            Event::Tick => match self {
                Toast::Error { ticks, .. } => {
                    if *ticks > TICK_FPS as u32 * 5 {
                        *self = Toast::Hidden;
                        EventState::Consumed
                    } else {
                        *ticks += 1;
                        EventState::Consumed
                    }
                }
                Toast::Loading { ticks, .. } => {
                    *ticks += 1;
                    EventState::Consumed
                }
                Toast::Hidden => EventState::NotConsumed,
            },
            Event::Keyboard(_) => EventState::NotConsumed,
            Event::StartLoadingItem => EventState::NotConsumed,
            Event::LoadedItem(_) => EventState::NotConsumed,
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        if self.hidden() {
            return;
        }

        let area = frame.area();

        let width = 30;
        let height = 4;

        let x = area.width - width - 2;
        let y = area.height - height - 1;

        let area = Rect::new(x, y, width, height);
        frame.render_widget(Clear, area);

        let paragraph = match self {
            Toast::Loading { message, ticks } => {
                let ch = spinner_frame(*ticks as usize);
                Paragraph::new(format!("{ch} {message}"))
            }
            Toast::Error { error, .. } => Paragraph::new(error.to_string()),
            Toast::Hidden => unreachable!(),
        };

        let paragraph =
            paragraph.block(Block::bordered().border_type(ratatui::widgets::BorderType::Rounded));
        frame.render_widget(paragraph, area);
    }

    fn hidden(&self) -> bool {
        matches!(self, Toast::Hidden)
    }
}

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Stylize},
    widgets::{Block, BorderType, Clear, Paragraph},
};

use crate::event::{Event, EventState, ToastEvent};

use super::spinner_frame;

#[derive(Default)]
enum ToastState {
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

pub struct Toast {
    state: ToastState,
    tick_fps: u32,
}

impl Toast {
    pub fn new(tick_fps: u32) -> Self {
        Self {
            state: ToastState::default(),
            tick_fps,
        }
    }

    pub fn handle_event(&mut self, event: &Event) -> EventState {
        match event {
            Event::Toast(ToastEvent::Loading(msg)) => {
                self.state = ToastState::Loading {
                    message: msg.to_string(),
                    ticks: 0,
                };
                EventState::Handled
            }
            Event::Toast(ToastEvent::Error(msg)) => {
                self.state = ToastState::Error {
                    error: msg.to_string(),
                    ticks: 0,
                };
                EventState::Handled
            }
            Event::Toast(ToastEvent::Hide) => {
                self.state = ToastState::Hidden;
                EventState::Handled
            }
            Event::Tick => match &mut self.state {
                ToastState::Error { ticks, .. } => {
                    if *ticks > self.tick_fps * 5 {
                        self.state = ToastState::Hidden;
                    } else {
                        *ticks += 1;
                    }

                    EventState::Handled
                }
                ToastState::Loading { ticks, .. } => {
                    *ticks += 1;
                    EventState::Handled
                }
                ToastState::Hidden => EventState::Ignored,
            },
            Event::Keyboard(_) => EventState::Ignored,
            Event::StartLoadingItem => EventState::Ignored,
            Event::LoadedItem(_) => EventState::Ignored,
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        if self.hidden() {
            return;
        }

        let area = frame.area();

        let width = 30;
        let height = 3;

        let x = area.width - width - 2;
        let y = area.height - height - 1;

        let area = Rect::new(x, y, width, height);
        frame.render_widget(Clear, area);

        let color = match &self.state {
            ToastState::Loading { .. } => Color::Cyan,
            ToastState::Error { .. } => Color::Red,
            ToastState::Hidden => unreachable!(),
        };

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(color);
        frame.render_widget(block, area);

        let paragraph = match &self.state {
            ToastState::Loading { message, ticks } => {
                let ch = spinner_frame(*ticks as usize);
                Paragraph::new(format!("{ch} {message}"))
            }
            ToastState::Error { error, .. } => Paragraph::new(error.to_string()),
            ToastState::Hidden => unreachable!(),
        };

        frame.render_widget(
            paragraph.style(color).bold(),
            Rect::new(x + 2, y + 1, width - 4, height - 2),
        );
    }

    fn hidden(&self) -> bool {
        matches!(self.state, ToastState::Hidden)
    }
}

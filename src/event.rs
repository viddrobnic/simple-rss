use std::time::Duration;

use crossterm::event::{Event as CrosstermEvent, KeyCode};
use futures::{FutureExt, StreamExt};
use simple_rss_lib::event::{Event, EventSender, KeyboardEvent};

pub const TICK_FPS: f64 = 30.0;

/// A thread that handles reading crossterm events and emitting tick events on a regular schedule.
pub struct EventTask {
    sender: EventSender,
}

impl EventTask {
    pub fn new(sender: EventSender) -> Self {
        Self { sender }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let tick_rate = Duration::from_secs_f64(1.0 / TICK_FPS);
        let mut tick = tokio::time::interval(tick_rate);
        let mut reader = crossterm::event::EventStream::new();
        loop {
            let tick_delay = tick.tick();
            let crossterm_event = reader.next().fuse();
            tokio::select! {
              _ = self.sender.closed() => {
                break;
              }
              _ = tick_delay => {
                self.sender.send(Event::Tick);
              }
              Some(Ok(evt)) = crossterm_event => {
                if let CrosstermEvent::Key(key_evt) = evt {
                    send_keycode(key_evt.code, &self.sender);
                }
              }
            };
        }
        Ok(())
    }
}

fn send_keycode(code: KeyCode, sender: &EventSender) {
    let event = match code {
        KeyCode::Left | KeyCode::Char('h') => KeyboardEvent::Left,
        KeyCode::Right | KeyCode::Char('l') => KeyboardEvent::Right,
        KeyCode::Up | KeyCode::Char('k') => KeyboardEvent::Up,
        KeyCode::Down | KeyCode::Char('j') => KeyboardEvent::Down,
        KeyCode::Esc | KeyCode::Char('q') => KeyboardEvent::Back,
        KeyCode::Enter => KeyboardEvent::Enter,
        KeyCode::Char(' ') => KeyboardEvent::Space,
        KeyCode::Char('o') => KeyboardEvent::Open,
        KeyCode::Char('?') => KeyboardEvent::Help,
        _ => return,
    };

    sender.send(Event::Keyboard(event));
}

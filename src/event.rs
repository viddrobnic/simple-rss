use std::time::Duration;

use crossterm::event::{Event as CrosstermEvent, KeyCode};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;

pub const TICK_FPS: f64 = 30.0;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Tick,
    Keyboard(KeyboardEvent),

    StartLoadingItem,
    LoadedItem(String),

    Toast(ToastEvent),
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum KeyboardEvent {
    Left,
    Right,
    Up,
    Down,
    Back,
    Enter,
    Space,
    Open,
    Help,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToastEvent {
    Loading(String),
    Error(String),
    Hide,
}

/// State of weather event has been handled.
/// If event is handled, it's still sent to other components.
/// It's mostly used to decide when to render the components
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EventState {
    Handled,
    Ignored,
}

impl EventState {
    pub fn is_handled(&self) -> bool {
        *self == Self::Handled
    }

    pub fn or(&self, other: &Self) -> Self {
        if self.is_handled() || other.is_handled() {
            Self::Handled
        } else {
            Self::Ignored
        }
    }
}

#[derive(Debug, Clone)]
pub struct EventSender(mpsc::UnboundedSender<Event>);

#[derive(Debug)]
pub struct EventHandler {
    sender: EventSender,
    receiver: mpsc::UnboundedReceiver<Event>,
}

impl EventSender {
    pub fn send(&self, event: Event) {
        let _ = self.0.send(event);
    }

    pub async fn closed(&self) {
        self.0.closed().await
    }
}

impl EventHandler {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let sender = EventSender(sender);

        let actor = EventTask::new(sender.clone());
        tokio::spawn(async { actor.run().await });

        Self { sender, receiver }
    }

    pub async fn next(&mut self) -> anyhow::Result<Event> {
        self.receiver
            .recv()
            .await
            .ok_or(anyhow::anyhow!("Failed to read event"))
    }

    pub fn get_sender(&self) -> EventSender {
        self.sender.clone()
    }
}

/// A thread that handles reading crossterm events and emitting tick events on a regular schedule.
struct EventTask {
    sender: EventSender,
}

impl EventTask {
    fn new(sender: EventSender) -> Self {
        Self { sender }
    }

    async fn run(self) -> anyhow::Result<()> {
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

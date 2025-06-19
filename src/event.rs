use std::time::Duration;

use crossterm::event::{Event as CrosstermEvent, KeyEvent};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;

pub const TICK_FPS: f64 = 30.0;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Tick,
    Keyboard(KeyEvent),

    StartLoadingItem,
    LoadedItem(String),

    ToastLoading(String),
    ToastError(String),
    ToastHide,
}

#[derive(Debug, Clone)]
pub struct EventSender(mpsc::UnboundedSender<Event>);

#[derive(Debug)]
pub struct EventHandler {
    sender: EventSender,
    receiver: mpsc::UnboundedReceiver<Event>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum EventState {
    Consumed,
    NotConsumed,
}

impl EventSender {
    pub fn send(&self, event: Event) {
        let _ = self.0.send(event);
    }

    pub async fn closed(&self) {
        self.0.closed().await
    }
}

impl EventState {
    pub fn is_consumed(&self) -> bool {
        *self == Self::Consumed
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
                    self.sender.send(Event::Keyboard(key_evt));
                }
              }
            };
        }
        Ok(())
    }
}

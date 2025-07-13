use tokio::sync::mpsc;

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

/// Send events to event bus.
#[derive(Debug, Clone)]
pub struct EventSender(mpsc::UnboundedSender<Event>);

impl EventSender {
    pub fn send(&self, event: Event) {
        let _ = self.0.send(event);
    }

    pub async fn closed(&self) {
        self.0.closed().await
    }
}

/// Handles sending of events
pub struct EventBus {
    sender: EventSender,
    receiver: mpsc::UnboundedReceiver<Event>,
}

impl Default for EventBus {
    fn default() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let sender = EventSender(sender);

        Self { sender, receiver }
    }
}

impl EventBus {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the next event. If channel has been closed, None is returned.
    /// If no event is buffered, it sleeps until the next event is available.
    pub async fn next(&mut self) -> Option<Event> {
        self.receiver.recv().await
    }

    pub fn get_sender(&self) -> EventSender {
        self.sender.clone()
    }
}

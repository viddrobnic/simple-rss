use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

use crate::{
    components::{Content, ItemList},
    data::DataLoader,
    event::{Event, EventSender, EventState},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Focus {
    ItemList,
    Content,
}

pub struct App {
    data_loader: DataLoader,

    focus: Focus,

    item_list: ItemList,
    content: Content,
}

impl App {
    pub fn new(event_tx: EventSender) -> anyhow::Result<Self> {
        let data_loader = DataLoader::new(event_tx.clone())?;

        Ok(Self {
            item_list: ItemList::new(true, event_tx, data_loader.clone()),
            content: Content::new(false),
            focus: Focus::ItemList,
            data_loader,
        })
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
            .spacing(1)
            .split(frame.area());

        self.item_list.draw(frame, layout[0]);
        self.content.draw(frame, layout[1]);
    }

    pub fn handle_event(&mut self, event: &Event) -> EventState {
        // Component events
        let state = self.item_list.handle_event(event);
        if state.is_consumed() {
            return EventState::Consumed;
        }

        let state = self.content.handle_event(event);
        if state.is_consumed() {
            return EventState::Consumed;
        }

        // Move focus
        match event {
            Event::Keyboard(key) => {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    match self.focus {
                        Focus::ItemList => EventState::NotConsumed,
                        Focus::Content => {
                            self.focus = Focus::ItemList;
                            self.item_list.set_focused(true);
                            self.content.set_focused(false);
                            EventState::Consumed
                        }
                    }
                } else {
                    EventState::NotConsumed
                }
            }
            Event::StartLoadingItem => match self.focus {
                Focus::ItemList => {
                    self.focus = Focus::Content;
                    self.item_list.set_focused(false);
                    self.content.set_focused(true);
                    EventState::Consumed
                }
                Focus::Content => EventState::NotConsumed,
            },
            Event::Tick => EventState::NotConsumed,
            Event::LoadedItem(_) => EventState::NotConsumed,
        }
    }
}

use crossterm::event::KeyCode;
use ratatui::widgets::ListState;

#[derive(Default)]
pub struct AppState {
    running: bool,
    items_state: ListState,
    active: ActiveState,
}

/// Returned by handle event function on a state type.
/// This determines how the event should be handled by the parent.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EventBehavior {
    Handle,
    Ignore,
}

impl AppState {
    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn set_running(&mut self) {
        self.running = true;
    }

    pub fn items_state_mut(&mut self) -> &mut ListState {
        &mut self.items_state
    }

    pub fn handle_event(&mut self, event: KeyCode) {
        let beh = match self.active {
            ActiveState::ItemsList => self.handle_items_list_event(event),
        };

        if beh == EventBehavior::Ignore {
            return;
        }

        if KeyCode::Char('q') == event {
            self.running = false;
        }
    }

    fn handle_items_list_event(&mut self, event: KeyCode) -> EventBehavior {
        match event {
            KeyCode::Down | KeyCode::Char('j') => {
                self.items_state.select_next();
                EventBehavior::Ignore
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.items_state.select_previous();
                EventBehavior::Ignore
            }
            _ => EventBehavior::Handle,
        }
    }
}

#[derive(Default)]
pub enum ActiveState {
    #[default]
    ItemsList,
}

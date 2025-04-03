use crossterm::event::Event;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
};

use crate::{
    data::{Channel, Data, Item},
    event::EventHandler,
    state::AppState,
    widget::{Content, ContentState, ItemList},
};

pub struct App {
    data: Data,

    state: AppState,
    events: EventHandler,
}

impl App {
    pub fn new() -> Self {
        Self {
            data: Data {
                channels: vec![
                    Channel {
                        title: "Test".to_string(),
                        description: "Test description 123".to_string(),
                    };
                    10
                ],
                items: vec![
                    Item {
                        id: "".to_string(),
                        title: "title".to_string(),
                        description: Some("very very long string asdf asdf asdf asdf asdf asdf asdf asdf asd fasdf asdf asdf asdf asdf asdf asdf asdf asdf asdf asdf asdf asdf asdf asf asdf ".to_string()),
                        link: None,
                        read: false,
                    };
                    50
                ],
            },
            state: AppState::default(),
            events: EventHandler::new(),
        }
    }

    pub async fn run(&mut self, mut terminal: DefaultTerminal) -> anyhow::Result<()> {
        self.state.set_running();
        while self.state.is_running() {
            terminal.draw(|t| self.render(t))?;
            match self.events.next().await? {
                crate::event::Event::Tick => self.tick(),
                crate::event::Event::Crossterm(event) => {
                    if let Event::Key(key_event) = event {
                        self.handle_keyboard(key_event)
                    }
                }
            }
        }

        Ok(())
    }

    fn tick(&mut self) {}

    fn handle_keyboard(&mut self, event: crossterm::event::KeyEvent) {
        self.state.handle_event(event.code);
    }

    fn render(&mut self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
            .spacing(1)
            .split(frame.area());

        let item_list = ItemList::new(
            self.state.is_items_list_active(),
            &self.data.items,
            self.state.items_state_mut(),
        );
        frame.render_widget(item_list, layout[0]);

        let content = Content::new(self.state.is_content_active(), self.state.content_state());
        frame.render_widget(content, layout[1]);
    }
}

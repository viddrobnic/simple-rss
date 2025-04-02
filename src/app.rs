use crossterm::event::{self, Event};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
};

use crate::{
    data::{Channel, Data, Item},
    widget::ItemList,
};

pub struct App {
    data: Data,
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
                    10
                ],
            },
        }
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> anyhow::Result<()> {
        loop {
            terminal.draw(|t| self.render(t))?;
            if matches!(event::read()?, Event::Key(_)) {
                break Ok(());
            }
        }
    }

    fn render(&self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
            .split(frame.area());

        frame.render_widget(ItemList::new(&self.data.items), layout[0]);
    }
}

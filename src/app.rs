use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
};

use crate::{
    data::{Channel, Data, DataLoader, Item},
    event::{Event, EventHandler},
    state::{AppState, EventBehavior},
    widget::{Content, ItemList},
};

pub struct App {
    data: Data,
    data_loader: DataLoader,

    state: AppState,
    events: EventHandler,
}

impl App {
    pub fn new() -> Self {
        let events = EventHandler::new();
        let data_loader = DataLoader::new(events.get_sender());

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
                        link: Some("https://viddrobnic.com/blog/2025/writing-my-language-3/".to_string()),
                        read: false,
                    };
                    50
                ],
            },
            state: AppState::new(data_loader.clone()),
            data_loader,
            events,
        }
    }

    pub async fn run(&mut self, mut terminal: DefaultTerminal) -> anyhow::Result<()> {
        self.state.set_running();
        while self.state.is_running() {
            terminal.draw(|t| self.render(t))?;
            let event = self.events.next().await?;
            self.handle_event(event);
        }

        Ok(())
    }

    fn handle_event(&mut self, event: Event) {
        let beh = self.state.handle_event(event);
        let EventBehavior::Handle(event) = beh else {
            return;
        };

        if let Event::StartLoadingItem(idx) = event {
            let Some(link) = &self.data.items[idx].link else {
                return;
            };

            let dl = self.data_loader.clone();
            let link = link.clone();
            tokio::spawn(async move {
                dl.load_item(&link).await;
            });
        }
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

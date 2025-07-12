use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

use crate::{
    components::{Content, Help, ItemList, Toast},
    data::DataLoader,
    event::{Event, EventSender, EventState, KeyboardEvent},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Focus {
    ItemList,
    Content,
    Help,
}

pub struct App {
    focus: Focus,

    // Focus before help is opened
    prev_focus: Option<Focus>,

    item_list: ItemList,
    content: Content,
    toast: Toast,
    help: Help,
}

impl App {
    pub fn new(event_sender: EventSender, data_loader: DataLoader) -> anyhow::Result<Self> {
        // Start refreshing
        let mut loader = data_loader.clone();
        tokio::spawn(async move { loader.refresh().await });

        Ok(Self {
            focus: Focus::ItemList,
            prev_focus: None,
            item_list: ItemList::new(true, event_sender, data_loader.clone()),
            content: Content::new(false),
            toast: Toast::new(),
            help: Help::new(),
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
        self.help.draw(frame);
        self.toast.draw(frame);
    }

    pub fn handle_event(&mut self, event: &Event) -> EventState {
        // Component events
        let mut res_state = self.item_list.handle_event(event);

        let state = self.content.handle_event(event);
        res_state = res_state.or(&state);

        let state = self.toast.handle_event(event);
        res_state = res_state.or(&state);

        // Move focus
        let state = match event {
            Event::Keyboard(key) => match key {
                KeyboardEvent::Back => match self.focus {
                    Focus::ItemList => EventState::Ignored,
                    Focus::Content => {
                        self.set_focus(Focus::ItemList);
                        EventState::Handled
                    }
                    Focus::Help => {
                        self.set_focus(self.prev_focus.unwrap_or(Focus::ItemList));
                        EventState::Handled
                    }
                },
                KeyboardEvent::Left => match self.focus {
                    Focus::Content => {
                        self.set_focus(Focus::ItemList);
                        EventState::Handled
                    }
                    Focus::ItemList | Focus::Help => EventState::Ignored,
                },
                KeyboardEvent::Right => match self.focus {
                    Focus::ItemList => {
                        self.set_focus(Focus::Content);
                        EventState::Handled
                    }
                    Focus::Content | Focus::Help => EventState::Ignored,
                },
                KeyboardEvent::Help => {
                    self.set_focus(Focus::Help);
                    EventState::Handled
                }
                _ => EventState::Ignored,
            },
            Event::StartLoadingItem => match self.focus {
                Focus::ItemList => {
                    self.set_focus(Focus::Content);
                    EventState::Handled
                }
                Focus::Content | Focus::Help => EventState::Ignored,
            },
            Event::Tick => EventState::Ignored,
            Event::LoadedItem(_) => EventState::Ignored,
            Event::Toast(_) => EventState::Ignored,
        };

        res_state.or(&state)
    }

    fn set_focus(&mut self, focus: Focus) {
        match focus {
            Focus::ItemList => {
                self.item_list.set_focused(true);
                self.content.set_focused(false);
                self.help.close();
            }
            Focus::Content => {
                self.item_list.set_focused(false);
                self.content.set_focused(true);
                self.help.close();
            }
            Focus::Help => {
                self.item_list.set_focused(false);
                self.content.set_focused(false);
                self.prev_focus = Some(self.focus);
                self.help.open();
            }
        }

        self.focus = focus;
    }
}

// use crossterm::event::KeyCode;
// use ratatui::widgets::ListState;
//
// use crate::{components::ContentState, data::DataLoader, event::Event};
//
// pub struct AppState {
//     running: bool,
//
//     data_loader: DataLoader,
//
//     active: ActiveState,
//
//     items_state: ListState,
//     content_state: ContentState,
// }
//
// impl AppState {
//     pub fn new(data_loader: DataLoader) -> Self {
//         Self {
//             running: false,
//             data_loader,
//             active: ActiveState::default(),
//             items_state: ListState::default(),
//             content_state: ContentState::default(),
//         }
//     }
//
//     pub fn is_running(&self) -> bool {
//         self.running
//     }
//
//     pub fn set_running(&mut self) {
//         self.running = true;
//     }
//
//     pub fn items_state_mut(&mut self) -> &mut ListState {
//         &mut self.items_state
//     }
//
//     pub fn is_items_list_active(&self) -> bool {
//         matches!(self.active, ActiveState::ItemsList)
//     }
//
//     pub fn is_content_active(&self) -> bool {
//         matches!(self.active, ActiveState::Content)
//     }
//
//     pub fn content_state(&self) -> &ContentState {
//         &self.content_state
//     }
//
//     pub fn handle_event(&mut self, event: Event) -> EventBehavior {
//         let beh = match self.active {
//             ActiveState::ItemsList => self.handle_items_list_event(event),
//             ActiveState::Content => self.handle_content_event(event),
//         };
//
//         let EventBehavior::Handle(event) = beh else {
//             return EventBehavior::Ignore;
//         };
//
//         match event {
//             Event::Keyboard(key_event) if key_event.code == KeyCode::Char('q') => {
//                 self.running = false;
//                 EventBehavior::Ignore
//             }
//             Event::Tick => {
//                 if let ContentState::Loading(t) = self.content_state {
//                     let (t, _) = t.overflowing_add(1);
//                     self.content_state = ContentState::Loading(t);
//                 }
//
//                 EventBehavior::Ignore
//             }
//             Event::LoadedItem(text) => {
//                 self.content_state = ContentState::Data(text);
//                 EventBehavior::Ignore
//             }
//             Event::Keyboard(_) => EventBehavior::Ignore,
//             e => EventBehavior::Handle(e),
//         }
//     }
//
//     fn handle_items_list_event(&mut self, event: Event) -> EventBehavior {
//         let Event::Keyboard(key_event) = event else {
//             return EventBehavior::Handle(event);
//         };
//
//         match key_event.code {
//             KeyCode::Down | KeyCode::Char('j') => {
//                 self.items_state.select_next();
//                 EventBehavior::Ignore
//             }
//             KeyCode::Up | KeyCode::Char('k') => {
//                 self.items_state.select_previous();
//                 EventBehavior::Ignore
//             }
//             KeyCode::Char(' ') | KeyCode::Enter => {
//                 let Some(selected) = self.items_state.selected() else {
//                     return EventBehavior::Ignore;
//                 };
//
//                 self.active = ActiveState::Content;
//                 self.content_state = ContentState::Loading(0);
//
//                 EventBehavior::Handle(Event::StartLoadingItem(selected))
//             }
//             _ => EventBehavior::Handle(Event::Keyboard(key_event)),
//         }
//     }
//
//     fn handle_content_event(&mut self, event: Event) -> EventBehavior {
//         let Event::Keyboard(key_event) = event else {
//             return EventBehavior::Handle(event);
//         };
//
//         match key_event.code {
//             KeyCode::Char('q') | KeyCode::Esc => {
//                 self.active = ActiveState::ItemsList;
//                 EventBehavior::Ignore
//             }
//             _ => EventBehavior::Handle(Event::Keyboard(key_event)),
//         }
//     }
// }
//
// #[derive(Default)]
// pub enum ActiveState {
//     #[default]
//     ItemsList,
//     Content,
// }

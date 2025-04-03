use ratatui::{
    style::Color,
    widgets::{Block, BorderType, Paragraph, Widget},
};

#[derive(Default)]
pub enum ContentState {
    #[default]
    Empty,
    Loading,
}

pub struct Content<'a> {
    selected: bool,
    state: &'a ContentState,
}

impl<'a> Content<'a> {
    pub fn new(selected: bool, state: &'a ContentState) -> Self {
        Self { selected, state }
    }

    fn render_empty(self, mut area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let block = basic_block(self.selected);
        block.render(area, buf);

        let paragraph = Paragraph::new("Select an item to get started").centered();

        area.y = area.height / 2;
        paragraph.render(area, buf);
    }

    fn render_loading(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let block = basic_block(self.selected);
        block.render(area, buf);
    }
}

fn basic_block(selected: bool) -> Block<'static> {
    let mut block = Block::bordered().border_type(BorderType::Rounded);
    if !selected {
        block = block.border_style(Color::Gray);
    }

    block
}

impl Widget for Content<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        match self.state {
            ContentState::Empty => self.render_empty(area, buf),
            ContentState::Loading => self.render_loading(area, buf),
        }
    }
}

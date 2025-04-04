use html2text::from_read;
use ratatui::{
    style::Color,
    widgets::{Block, BorderType, Paragraph, Widget, Wrap},
};

#[derive(Default)]
pub enum ContentState {
    #[default]
    Empty,
    Loading(u8),
    Data(String),
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

    fn render_loading(
        self,
        mut area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        tick: u8,
    ) {
        let block = basic_block(self.selected);
        block.render(area, buf);

        let paragraph = Paragraph::new(format!("Loading {tick}")).centered();

        area.y = area.height / 2;
        paragraph.render(area, buf);
    }

    fn render_data(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        text: &str,
    ) {
        let block = basic_block(self.selected);

        let text =
            from_read(text.as_bytes(), area.width as usize).unwrap_or_else(|_| text.to_string());

        let paragraph = Paragraph::new(text)
            .block(block)
            .wrap(Wrap { trim: true })
            .scroll((0, 0));

        paragraph.render(area, buf);
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
            ContentState::Loading(tick) => self.render_loading(area, buf, *tick),
            ContentState::Data(text) => self.render_data(area, buf, text),
        }
    }
}

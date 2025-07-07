use ratatui::{
    Frame,
    layout::Rect,
    style::Stylize,
    widgets::{Block, BorderType, Clear, Paragraph},
};

const SPACING: u16 = 3;
const NR_ENTRIES: u16 = 6;

pub struct Help {
    open: bool,
    keys: Paragraph<'static>,
    descs: Paragraph<'static>,

    keys_width: u16,
    descs_width: u16,
}

impl Help {
    pub fn new() -> Self {
        Self {
            open: false,
            keys: Paragraph::new(vec![
                "<Enter>".into(),
                "<Esc> / <q>".into(),
                "<o>".into(),
                "<Space>".into(),
                "<Up> / <Down> / <j> / <k>".into(),
                "<Left> / <Right> / <h> / <l>".into(),
            ])
            .centered()
            .blue()
            .bold(),
            descs: Paragraph::new(vec![
                "Select".into(),
                "Go Back / Exit".into(),
                "Open in browser".into(),
                "Mark/Unmark item in list as read".into(),
                "Scroll up / down".into(),
                "Change focus between item list and content".into(),
            ]),

            keys_width: 28,
            descs_width: 42,
        }
    }

    pub fn open(&mut self) {
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn draw(&self, frame: &mut Frame) {
        if !self.open {
            return;
        }

        let width = self.keys_width + self.descs_width + SPACING + 2 + 2; // 2 border + 2 space
        let height = NR_ENTRIES + 2 + 1; // 2  border + 1  title
        let area = Rect::new(
            (frame.area().width - width) / 2,
            (frame.area().height - height) / 2,
            width,
            height,
        );
        frame.render_widget(Clear, area);

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title("Help");
        frame.render_widget(block, area);

        frame.render_widget(
            Paragraph::new("Key:").centered().bold(),
            Rect::new(area.x + 2, area.y + 1, self.keys_width, 1),
        );
        frame.render_widget(
            &self.keys,
            Rect::new(area.x + 2, area.y + 2, self.keys_width, NR_ENTRIES),
        );

        frame.render_widget(
            Paragraph::new("Description:").bold(),
            Rect::new(
                area.x + 2 + self.keys_width + SPACING,
                area.y + 1,
                self.descs_width,
                1,
            ),
        );
        frame.render_widget(
            &self.descs,
            Rect::new(
                area.x + 2 + self.keys_width + SPACING,
                area.y + 2,
                self.descs_width,
                NR_ENTRIES,
            ),
        );
    }
}

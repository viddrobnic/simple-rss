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
    pub fn new(disable_read_status: bool, disable_browser_open: bool) -> Self {
        let (keys, descs) = build_paragraph(disable_read_status, disable_browser_open);
        Self {
            open: false,
            keys,
            descs,
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

fn build_paragraph(
    disable_read_status: bool,
    disable_browser_open: bool,
) -> (Paragraph<'static>, Paragraph<'static>) {
    let mut keys = vec!["<Enter>".into(), "<Esc> / <q>".into()];
    if !disable_browser_open {
        keys.push("<o>".into());
    }
    if !disable_read_status {
        keys.push("<Space>".into());
    }
    keys.extend_from_slice(&[
        "<Up> / <Down> / <j> / <k>".into(),
        "<Left> / <Right> / <h> / <l>".into(),
    ]);
    let keys = Paragraph::new(keys).centered().blue().bold();

    let mut descs = vec!["Select".into(), "Go Back / Exit".into()];
    if !disable_browser_open {
        descs.push("Open in browser".into());
    }
    if !disable_read_status {
        descs.push("Mark/Unmark item in list as read".into());
    }
    descs.extend_from_slice(&[
        "Scroll up / down".into(),
        "Change focus between item list and content".into(),
    ]);
    let descs = Paragraph::new(descs);

    (keys, descs)
}

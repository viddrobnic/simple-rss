use app::App;

mod app;
mod data;
mod widget;

fn main() -> anyhow::Result<()> {
    let terminal = ratatui::init();
    let mut app = App::new();
    let result = app.run(terminal);
    ratatui::restore();
    result
}

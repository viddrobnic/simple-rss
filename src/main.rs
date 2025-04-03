use app::App;

mod app;
mod data;
mod event;
mod state;
mod widget;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let terminal = ratatui::init();
    let mut app = App::new();
    let result = app.run(terminal).await;
    ratatui::restore();
    result
}

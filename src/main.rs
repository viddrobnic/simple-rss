use app::App;
use crossterm::event::KeyCode;
use event::{Event, EventHandler};

mod app;
mod components;
mod data;
mod event;
mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut terminal = ratatui::init();

    let mut events = EventHandler::new();
    let mut app = App::new(events.get_sender());

    loop {
        terminal.draw(|f| app.draw(f))?;

        let event = events.next().await?;
        let state = app.handle_event(&event);
        if state.is_consumed() {
            continue;
        }

        let Event::Keyboard(key) = event else {
            continue;
        };

        if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
            break;
        }
    }

    ratatui::restore();
    Ok(())
}

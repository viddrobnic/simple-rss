use app::App;
use clap::{Parser, Subcommand};
use crossterm::event::KeyCode;
use data::DataLoader;
use event::{Event, EventHandler};

mod app;
mod components;
mod data;
mod event;
mod html_render;
mod path;
mod state;

#[derive(Debug, Parser)]
#[command(version, about, long_about)]
/// Simple RSS Reader
///
/// Run without arguments to run the main TUI.
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Manage channels
    #[clap(visible_alias = "ch")]
    Channel {
        #[command(subcommand)]
        command: ChannelCommands,
    },
}

#[derive(Debug, Subcommand)]
enum ChannelCommands {
    /// List channels
    #[clap(visible_alias = "ls")]
    List,

    /// Add a new channel
    Add {
        /// URL of the feed
        url: String,

        /// Custom name for the feed
        #[arg(long)]
        name: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        None => run().await,
        Some(Commands::Channel { command }) => manage_channel(command),
    }
}

async fn run() -> anyhow::Result<()> {
    let mut terminal = ratatui::init();

    let mut events = EventHandler::new();
    let data_loader = DataLoader::new(events.get_sender())?;
    let mut app = App::new(events.get_sender(), data_loader.clone())?;

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
            data_loader.save()?;
            break;
        }
    }

    ratatui::restore();
    Ok(())
}

fn manage_channel(cmd: ChannelCommands) -> anyhow::Result<()> {
    match cmd {
        ChannelCommands::List => todo!(),
        ChannelCommands::Add { url, name } => todo!(),
    }
}

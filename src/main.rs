use app::App;
use clap::{Parser, Subcommand};
use colored::{ColoredString, Colorize};
use crossterm::event::KeyCode;
use data::{Channel, Data, DataLoader};
use event::{Event, EventHandler};
use unicode_width::UnicodeWidthStr;

mod app;
mod components;
mod data;
mod event;
mod html_render;
mod path;
mod state;

const NAME_TITLE: &str = "Name";
const URL_TITLE: &str = "URL";

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

    /// Remove a channel
    #[clap(visible_alias = "rm")]
    Remove {
        /// Index of the channel to remove.
        /// Run `simple-rss channel list` to see indices.
        idx: usize,
    },

    /// Edit a channel
    Edit {
        /// Index of the channel to remove.
        /// Run `simple-rss channel list` to see indices.
        idx: usize,

        /// Custom name for the feed
        #[arg(long)]
        name: Option<String>,

        /// URL of the feed
        #[arg(long)]
        url: Option<String>,
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
        ChannelCommands::List => list_channels(),
        ChannelCommands::Add { url, name } => add_channel(Channel { name, url }),
        ChannelCommands::Remove { idx } => remove_channel(idx),
        ChannelCommands::Edit { idx, name, url } => edit_channel(idx, name, url),
    }
}

fn add_channel(channel: Channel) -> anyhow::Result<()> {
    let mut data = Data::load()?;
    data.channels.push(channel);
    data.save()?;

    println!("✅ {}", "Channel added!".green().bold());

    Ok(())
}

fn remove_channel(idx: usize) -> anyhow::Result<()> {
    let mut data = Data::load()?;
    if idx >= data.channels.len() {
        println!("{}", "Invalid index!".yellow().bold());
        return Ok(());
    }

    data.channels.remove(idx);
    data.save()?;

    println!("✅ {}", "Channel removed!".green().bold());
    Ok(())
}

fn edit_channel(idx: usize, name: Option<String>, url: Option<String>) -> anyhow::Result<()> {
    if name.is_none() && url.is_none() {
        println!("{}", "Nothing to do!".bold());
        return Ok(());
    }

    let mut data = Data::load()?;
    if idx >= data.channels.len() {
        println!("{}", "Invalid index!".yellow().bold());
        return Ok(());
    }

    if name.is_some() {
        data.channels[idx].name = name;
    }
    if let Some(url) = url {
        data.channels[idx].url = url;
    }
    data.save()?;

    println!("✅ {}", "Channel updated!".green().bold());

    Ok(())
}

fn list_channels() -> anyhow::Result<()> {
    let data = Data::load()?;
    if data.channels.is_empty() {
        println!(
            "No channels added!\nRun `{}` to add a channel.",
            "simple-rss ch add".white()
        );
        return Ok(());
    }

    let (mut name_len, mut url_len) = data.channels.iter().fold((0, 0), |(n, u), it| {
        (
            n.max(it.name.as_ref().map_or(0, |v| v.width())),
            u.max(it.url.len()),
        )
    });

    if name_len < NAME_TITLE.len() {
        name_len = NAME_TITLE.len();
    }
    name_len += 2; // Space around

    if url_len < URL_TITLE.len() {
        url_len = URL_TITLE.len();
    }
    url_len += 1; // Space at the left

    // Print header
    print!("{} │", "idx".bold());
    print_center(name_len, NAME_TITLE.bold());
    print!("│");
    print_center(url_len, URL_TITLE.bold());
    println!();

    print!("────┼");
    for _ in 0..name_len {
        print!("─");
    }
    print!("┼");
    for _ in 0..url_len {
        print!("─");
    }
    println!();

    for (idx, ch) in data.channels.iter().enumerate() {
        print_channel(idx, ch, name_len);
    }

    Ok(())
}

fn print_channel(idx: usize, ch: &Channel, name_len: usize) {
    let idx = idx.to_string();
    print!("{}", idx.white());
    for _ in 0..(4 - idx.len()) {
        print!(" ")
    }
    print!("│ ");

    if let Some(name) = &ch.name {
        print!("{}", name);
    }

    let space = name_len - 1 - ch.name.as_ref().map_or(0, |n| n.width());
    for _ in 0..space {
        print!(" ");
    }
    print!("│ ");

    println!("{}", ch.url.blue());
}

fn print_center(len: usize, val: ColoredString) {
    let space = (len - val.chars().count()) / 2;
    for _ in 0..space {
        print!(" ");
    }
    print!("{}", val);
    let space = len - val.chars().count() - space;
    for _ in 0..space {
        print!(" ");
    }
}

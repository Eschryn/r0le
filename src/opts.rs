use clap::{AppSettings, Clap};

/// simple reaction role bot
#[derive(Clap)]
#[clap(version = "1.0", author = "Eschryn")]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    /// bot token - required to start the bot
    #[clap(short, long)]
    pub token: String,
    /// application id - required to start the bot
    #[clap(short, long)]
    pub application_id: u64,
    /// redis url
    #[clap(short, long)]
    pub redis_url: Option<String>
}

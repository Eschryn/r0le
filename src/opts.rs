use clap::{AppSettings, Clap};

/// simple reaction role bot
#[derive(Clap)]
#[clap(version = "1.0", author = "Eschryn")]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    /// bot token - required to start the bot
    #[clap(short, long)]
    pub token: Option<String>,
    /// application id - required to start the bot
    #[clap(short, long)]
    pub application_id: Option<u64>,
    /// redis url - required to start the bot
    #[clap(short, long)]
    pub redis_url: Option<String>
}

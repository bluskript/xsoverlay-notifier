use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};
use twelf::config;

#[derive(Serialize, Deserialize, Debug, EnumIter, Display, Clone, Copy, Default, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum NotificationStrategy {
    #[default]
    Listener,
    Polling,
}

#[derive(Debug, Clone, Default, Parser, Serialize)]
#[command(author, version, about, long_about = None)]
#[config]
pub struct NotifierConfig {
    #[arg(short, long, default_value_t = 42069)]
    pub port: usize,
    #[arg(long, default_value = "localhost")]
    pub host: String,
    #[arg(short, long, value_enum, default_value_t = NotificationStrategy::Listener)]
    pub notification_strategy: NotificationStrategy,
    #[arg(long, default_value_t = 250)]
    pub polling_rate: i32,
}

use std::path::Path;

use crate::tui::{start_tui, Tui};
use anyhow::Context;
use clap::CommandFactory;
use config::NotifierConfig;
use notif_handling::notification_listener;
use tokio::{
    fs::File,
    io::AsyncWriteExt,
    sync::{mpsc, watch},
};
use twelf::Layer;
use xsoverlay::xsoverlay_notifier;

pub mod config;
pub mod notif_handling;
pub mod tui;
pub mod xsoverlay;

async fn start() -> anyhow::Result<()> {
    tui_logger::init_logger(log::LevelFilter::Trace).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Trace);
    let matches = NotifierConfig::command().get_matches();
    if !Path::new("./config.toml").exists() {
        let mut file = File::create("./config.toml").await?;
        file.write_all(toml::to_string_pretty(&NotifierConfig::default())?.as_bytes())
            .await?;
    }
    let config = NotifierConfig::with_layers(&[
        Layer::Toml("./config.toml".into()),
        Layer::Env(Some("XSNOTIF_".into())),
        Layer::Clap(matches),
    ])
    .context("failed to parse config")?;
    let (tx, mut rx) = mpsc::unbounded_channel();
    let (config_tx, config_rx) = watch::channel(config);
    {
        let mut config_rx = config_rx.clone();
        tokio::spawn(async move {
            loop {
                let res = xsoverlay_notifier(&mut rx, &mut config_rx).await;
                log::error!(
                    "XSOverlay notification sender died unexpectedly: {:?}, restarting sender",
                    res
                );
            }
        });
    }
    tokio::task::spawn_blocking(|| {
        let local = tokio::task::LocalSet::new();
        tokio::runtime::Handle::current().block_on(local.run_until(async move {
            loop {
                let res = notification_listener(&tx).await;
                log::error!("Windows notification listener died unexpectedly: {:?}", res);
            }
        }));
    });
    let mut tui = Tui::default();
    start_tui(&mut tui, config_tx, config_rx).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    start().await
}

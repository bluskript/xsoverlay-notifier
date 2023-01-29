use anyhow::Context;
use clap::CommandFactory;
use config::NotifierConfig;
use directories::ProjectDirs;
use notif_handling::notification_listener;
use tokio::{
    fs::{create_dir_all, File},
    io::AsyncWriteExt,
    sync::mpsc,
};
use twelf::Layer;
use xsoverlay::xsoverlay_notifier;

pub mod config;
pub mod notif_handling;
pub mod xsoverlay;

async fn start() -> anyhow::Result<()> {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Debug)
        .init();
    let matches = NotifierConfig::command().get_matches();
    let project_dirs = ProjectDirs::from("dev", "blusk", "xsoverlay_notifier")
        .ok_or_else(|| anyhow::anyhow!("project dir lookup failed"))?;
    let config_file_path = project_dirs.config_dir().join("./config.toml");
    log::info!("checking if config file exists...");
    if !config_file_path.exists() {
        create_dir_all(project_dirs.config_dir()).await?;
        let mut file = File::create(config_file_path.clone()).await?;
        file.write_all(include_bytes!("./default_config.toml"))
            .await?;
        log::info!("default config written to {:?}", config_file_path);
    }
    let config = NotifierConfig::with_layers(&[
        Layer::Toml(config_file_path),
        Layer::Env(Some("XSNOTIF_".into())),
        Layer::Clap(matches),
    ])
    .context("failed to parse config")?;
    let (tx, mut rx) = mpsc::unbounded_channel();
    {
        let config = config.clone();
        tokio::spawn(async move {
            loop {
                let res = xsoverlay_notifier(&mut rx, &config.host, config.port).await;
                log::error!(
                    "XSOverlay notification sender died unexpectedly: {:?}, restarting sender",
                    res
                );
            }
        });
    }
    loop {
        let res = notification_listener(&config, &tx).await;
        log::error!("Windows notification listener died unexpectedly: {:?}", res);
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    start().await
}

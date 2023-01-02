use clap::Parser;
use notif_handling::notification_listener;
use std::io::stdin;
use tokio::sync::mpsc;
use xsoverlay::xsoverlay_notifier;

pub mod notif_handling;
pub mod xsoverlay;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 42069)]
    port: usize,
    #[arg(long, default_value = "localhost")]
    host: String,
}

async fn start() -> anyhow::Result<()> {
    let Args { host, port } = Args::parse();
    let (tx, mut rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        loop {
            let res = xsoverlay_notifier(&host, port, &mut rx).await;
            println!("XSOverlay notification worker died unexpectedly: {:?}", res);
            println!("Restarting worker");
        }
    });
    loop {
        let res = notification_listener(&tx).await;
        println!("Windows notification listener died unexpectedly: {:?}", res);
        println!("Restarting listener");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("{:?}", start().await);
    let mut buf = String::new();
    stdin().read_line(&mut buf)?;
    Ok(())
}

use anyhow::Context;
use miniserde::{json, Deserialize, Serialize};
use tokio::{net::UdpSocket, sync::mpsc};

#[derive(Serialize, Deserialize, Debug)]
pub struct XSOverlayMessage {
    /// 1 = Notification Popup, 2 = MediaPlayer Information, will be extended later on.
    pub messageType: i32,
    /// Only used for Media Player, changes the icon on the wrist.
    pub index: i32,
    /// How long the notification will stay on screen for in seconds
    pub timeout: f32,
    /// Height notification will expand to if it has content other than a title. Default is 175
    pub height: f32,
    /// Opacity of the notification, to make it less intrusive. Setting to 0 will set to 1.
    pub opacity: f32,
    /// Notification sound volume.
    pub volume: f32,
    /// File path to .ogg audio file. Can be "default", "error", or "warning". Notification will be silent if left empty.
    pub audioPath: String,
    /// Notification title, supports Rich Text Formatting
    pub title: String,
    /// Notification content, supports Rich Text Formatting, if left empty, notification will be small.
    pub content: String,
    /// Set to true if using Base64 for the icon image
    pub useBase64Icon: bool,
    /// Base64 Encoded image, or file path to image. Can also be "default", "error", or "warning"
    pub icon: String,
    /// Somewhere to put your app name for debugging purposes
    pub sourceApp: String,
}

pub async fn xsoverlay_notifier(
    host: &String,
    port: usize,
    rx: &mut mpsc::UnboundedReceiver<XSOverlayMessage>,
) -> anyhow::Result<()> {
    // using port 0 so the OS allocates a available port automatically
    let socket = UdpSocket::bind("0.0.0.0:0")
        .await
        .context("Failed to bind to local UDP port")?;
    socket
        .connect(format!("{host}:{port}"))
        .await
        .context("Failed to connect to XSOverlay Notification Daemon")?;
    while let Some(msg) = rx.recv().await {
        println!("Sending notification from {}", msg.sourceApp);
        let data = json::to_string(&msg);
        socket
            .send(data.as_bytes())
            .await
            .context("Failed to send notification to XSOverlay UDP socket")?;
    }
    Ok(())
}

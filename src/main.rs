use anyhow::{anyhow, Context};
use clap::Parser;
use miniserde::{json, Deserialize, Serialize};
use tokio::{
    net::UdpSocket,
    sync::mpsc::{self, unbounded_channel, UnboundedSender},
};
use windows::{
    Foundation::TypedEventHandler,
    UI::Notifications::{
        Management::{UserNotificationListener, UserNotificationListenerAccessStatus},
        Notification, UserNotification, UserNotificationChangedEventArgs,
        UserNotificationChangedKind,
    },
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 42069)]
    port: usize,
    #[arg(long, default_value = "localhost")]
    host: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct XSOverlayMessage {
    messageType: i32, // 1 = Notification Popup, 2 = MediaPlayer Information, will be extended later on.
    index: i32,       //Only used for Media Player, changes the icon on the wrist.
    timeout: f32,     //How long the notification will stay on screen for in seconds
    height: f32, //Height notification will expand to if it has content other than a title. Default is 175
    opacity: f32, //Opacity of the notification, to make it less intrusive. Setting to 0 will set to 1.
    volume: f32,  // Notification sound volume.
    audioPath: String, //File path to .ogg audio file. Can be "default", "error", or "warning". Notification will be silent if left empty.
    title: String,     //Notification title, supports Rich Text Formatting
    content: String, //Notification content, supports Rich Text Formatting, if left empty, notification will be small.
    useBase64Icon: bool, //Set to true if using Base64 for the icon image
    icon: String, //Base64 Encoded image, or file path to image. Can also be "default", "error", or "warning"
    sourceApp: String, //Somewhere to put your app name for debugging purposes
}

pub async fn xsoverlay_notifier(
    host: String,
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
        println!("Sending notification: {:?}", msg);
        let data = json::to_string(&XSOverlayMessage::default());
        socket.send(data.as_bytes()).await?;
    }
    Ok(())
}

pub async fn notif_to_message(notif: UserNotification) -> anyhow::Result<XSOverlayMessage> {
    let app_info = notif.AppInfo()?;
    let display_info = app_info.DisplayInfo()?;
    let app_name = display_info.DisplayName()?.to_string();
    let description = display_info.Description()?.to_string();
    Ok(XSOverlayMessage {
        messageType: 1,
        index: 0,
        timeout: 0.5,
        height: 175.,
        opacity: 1.,
        volume: 0.7,
        audioPath: "".to_string(),
        title: "".to_string(),
        content: "".to_string(),
        useBase64Icon: false,
        icon: "".to_string(),
        sourceApp: app_name,
    })
}

pub async fn notification_listener(tx: UnboundedSender<XSOverlayMessage>) -> anyhow::Result<()> {
    let listener = UserNotificationListener::Current()
        .context("failed to initialize user notification listener")?;
    println!("Requesting notification access");
    let access_status = listener
        .RequestAccessAsync()
        .context("Notification access request failed")?
        .await
        .context("Notification access request failed")?;
    if access_status != UserNotificationListenerAccessStatus::Allowed {
        return Err(anyhow!(
            "Notification access was not granted, was instead {:?}",
            access_status
        ));
    }
    println!("Notification access granted");

    let (new_notif_tx, mut new_notif_rx) = unbounded_channel::<u32>();
    listener
        .NotificationChanged(&TypedEventHandler::new(
            move |_sender, args: &Option<UserNotificationChangedEventArgs>| {
                println!("handling new notification event");
                if let Some(event) = args {
                    if event.ChangeKind()? == UserNotificationChangedKind::Added {
                        let id = event.UserNotificationId()?;
                        if let Err(e) = new_notif_tx.send(id) {
                            println!("Error sending ID of new notification: {e}");
                        }
                    };
                }
                Ok(())
            },
        ))
        .context("failed to register notification change handler")?;
    while let Some(notif_id) = new_notif_rx.recv().await {
        let notif = listener
            .GetNotification(notif_id)
            .context(format!("failed to get notification {notif_id}"))?;
        let msg = notif_to_message(notif).await;
        match msg {
            Ok(msg) => tx.send(msg)?,
            Err(e) => println!("Failed to convert notification to XSOverlay message: {e}"),
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Args { host, port } = Args::parse();
    let (tx, mut rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        xsoverlay_notifier(host, port, &mut rx).await.unwrap();
    });
    notification_listener(tx).await?;
    Ok(())
}

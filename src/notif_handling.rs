use anyhow::{anyhow, Context};
use base64::encode;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use windows::{
    ApplicationModel::AppDisplayInfo,
    Foundation::{GuidHelper, MemoryBuffer, Size, TypedEventHandler},
    Graphics::Imaging::BitmapEncoder,
    Storage::Streams::{
        Buffer, DataReader, IBuffer, IInputStream, IRandomAccessStreamWithContentType,
        InputStreamOptions,
    },
    UI::Notifications::{
        KnownNotificationBindings,
        Management::{UserNotificationListener, UserNotificationListenerAccessStatus},
        UserNotification, UserNotificationChangedEventArgs, UserNotificationChangedKind,
    },
};

use crate::xsoverlay::XSOverlayMessage;

async fn read_logo(display_info: AppDisplayInfo) -> anyhow::Result<Vec<u8>> {
    let logo_stream = display_info
        .GetLogo(Size {
            Width: 0.,
            Height: 0.,
        })
        .context("failed to get logo with size")?
        .OpenReadAsync()
        .context("failed to open for reading")?
        .await
        .context("awaiting opening for reading failed")?;
    read_stream_to_bytes(logo_stream)
        .await
        .context("failed to read stream to bytes")
}

pub async fn notif_to_message(notif: UserNotification) -> anyhow::Result<XSOverlayMessage> {
    let app_info = notif.AppInfo()?;
    let display_info = app_info.DisplayInfo()?;
    let app_name = display_info.DisplayName()?.to_string();
    let icon = read_logo(display_info)
        .await
        .map(encode)
        .unwrap_or_else(|err| {
            println!("{:?}", err.context("failed to read logo"));
            "default".to_string()
        });
    let toast_binding = notif
        .Notification()?
        .Visual()?
        .GetBinding(&KnownNotificationBindings::ToastGeneric()?)?;
    let text_elements = toast_binding.GetTextElements()?;
    // println!("{:?}", toast_binding.Template());
    // println!(
    //     "{:?}",
    //     toast_binding
    //         .Hints()?
    //         .into_iter()
    //         .map(|entry| (entry.Key(), entry.Value()))
    //         .collect::<Vec<_>>()
    // );
    let title = text_elements.GetAt(0)?.Text()?.to_string();
    let content = text_elements
        .into_iter()
        .skip(1)
        .map(|element| element.Text())
        .filter_map(|el| el.ok())
        .fold(String::new(), |a, b| a + &b.to_string() + "\n");
    Ok(XSOverlayMessage {
        messageType: 1,
        index: 0,
        timeout: 0.5,
        height: 175.,
        opacity: 1.,
        volume: 0.7,
        audioPath: "default".to_string(),
        title,
        content,
        useBase64Icon: true,
        icon,
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
                if let Some(event) = args {
                    if event.ChangeKind()? == UserNotificationChangedKind::Added {
                        println!("handling new notification event");
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

async fn read_stream_to_bytes(
    stream: IRandomAccessStreamWithContentType,
) -> anyhow::Result<Vec<u8>> {
    let stream_len = stream.Size()? as usize;
    let mut data = vec![0u8; stream_len];
    let reader = DataReader::CreateDataReader(&stream)?;
    reader.LoadAsync(stream_len as u32)?.await?;
    reader.ReadBytes(&mut data)?;
    reader.Close().ok();
    stream.Close().ok();
    Ok(data)
}

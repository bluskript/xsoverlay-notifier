use std::time::Duration;

use anyhow::Result;
use anyhow::{anyhow, Context};
use base64::encode;
use log::info;
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedSender},
    time::sleep,
};
use windows::{
    ApplicationModel::AppDisplayInfo,
    Foundation::{Size, TypedEventHandler},
    Storage::Streams::{DataReader, IRandomAccessStreamWithContentType},
    UI::Notifications::{
        KnownNotificationBindings,
        Management::{UserNotificationListener, UserNotificationListenerAccessStatus},
        NotificationKinds, UserNotification, UserNotificationChangedEventArgs,
        UserNotificationChangedKind,
    },
};

use crate::config::{NotificationStrategy, NotifierConfig};
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

pub async fn notif_to_message(
    notif: UserNotification,
    timeout: f32,
) -> anyhow::Result<XSOverlayMessage> {
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
        timeout,
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

pub async fn polling_notification_handler(
    listener: UserNotificationListener,
    tx: &UnboundedSender<XSOverlayMessage>,
    polling_rate: u64,
    timeout: f32,
) -> Result<()> {
    let mut prev_notifs: Option<Vec<UserNotification>> = None;
    loop {
        let notifs = listener
            .GetNotificationsAsync(NotificationKinds::Toast)?
            .await?;
        if let Some(prev_notifs) = prev_notifs {
            for notif in notifs.clone().into_iter().filter(|notif| {
                prev_notifs
                    .iter()
                    .find(|prev_notif| {
                        notif.Id().unwrap_or_default() == prev_notif.Id().unwrap_or_default()
                    })
                    .is_none()
            }) {
                log::info!("handling new notification");
                let msg = notif_to_message(notif, timeout).await;
                match msg {
                    Ok(msg) => tx.send(msg)?,
                    Err(e) => println!("Failed to convert notification to XSOverlay message: {e}"),
                }
            }
        }
        prev_notifs = Some(notifs.into_iter().collect::<Vec<UserNotification>>());
        sleep(Duration::from_millis(polling_rate)).await;
    }
}

pub async fn listening_notification_handler(
    listener: UserNotificationListener,
    tx: &UnboundedSender<XSOverlayMessage>,
    timeout: f32,
) -> Result<()> {
    let (new_notif_tx, mut new_notif_rx) = unbounded_channel::<u32>();
    listener
        .NotificationChanged(&TypedEventHandler::new(
            move |_sender, args: &Option<UserNotificationChangedEventArgs>| {
                if let Some(event) = args {
                    if event.ChangeKind()? == UserNotificationChangedKind::Added {
                        log::info!("handling new notification event");
                        let id = event.UserNotificationId()?;
                        if let Err(e) = new_notif_tx.send(id) {
                            log::error!("Error sending ID of new notification: {e}");
                        }
                    };
                }
                Ok(())
            },
        ))
        .context("failed to register notification change handler")?;
    while let Some(notif_id) = new_notif_rx.recv().await {
        if let Err(e) = async {
            let notif = listener
                .GetNotification(notif_id)
                .context(format!("failed to get notification {notif_id}"))?;
            let msg = notif_to_message(notif, timeout).await;
            match msg {
                Ok(msg) => tx.send(msg)?,
                Err(e) => println!("Failed to convert notification to XSOverlay message: {e}"),
            }
            anyhow::Ok(())
        }
        .await
        {
            log::error!("Failed to process notification: {e}");
        };
    }
    Ok(())
}

pub async fn notification_listener(
    config: &NotifierConfig,
    tx: &UnboundedSender<XSOverlayMessage>,
) -> anyhow::Result<()> {
    let listener = UserNotificationListener::Current()
        .context("failed to initialize user notification listener")?;
    info!("Requesting notification access");
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
    info!("Notification access granted");
    match config.notification_strategy {
        NotificationStrategy::Listener => {
            listening_notification_handler(listener, tx, config.timeout).await
        }
        NotificationStrategy::Polling => {
            polling_notification_handler(listener, tx, config.polling_rate, config.timeout).await
        }
    }
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

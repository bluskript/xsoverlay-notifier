# XSOverlay Notifier

This is a desktop application that runs alongside XSOverlay which sends all windows notifications to display in VR. This uses the [Windows Notification Listener API](https://learn.microsoft.com/en-us/windows/apps/design/shell/tiles-and-notifications/notification-listener) to listen for notifications.

One-liner to install and get it running:

```
Start-Process powershell.exe -Verb runas -ArgumentList '-Command iex ([System.Text.Encoding]::ASCII.GetString((iwr -UseBasicParsing -Uri https://github.com/bluskript/xsoverlay-notifier/releases/download/latest/install.ps1).Content))'
```

If you want to launch the notifier any time in the future, this adds an item to the Windows start menu as well so you should launch it from there.

## Configuration

Settings for the notifier are stored in `%APPDATA%\blusk\xsoverlay_notifier\config\config.toml`. Here is some brief documentation on each option in the config:

```toml
# Port that xsoverlay is listening on
port = 42069
# The hostname that xsoverlay is listening on
host = "localhost"
# The notification strategy - either "listener" or "polling"
notification_strategy = "polling"
# The rate at which the polling strategy refreshes notifications
polling_rate = 250
# The duration the notification shows up on screen
timeout = 2
```

https://user-images.githubusercontent.com/52386117/210190106-17c0cb01-8f35-4135-9db9-68f06e6400ec.mp4

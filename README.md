# XSOverlay Notifier

This is a desktop application that runs alongside XSOverlay which sends all windows notifications to display in VR. This uses the [Windows Notification Listener API](https://learn.microsoft.com/en-us/windows/apps/design/shell/tiles-and-notifications/notification-listener) to listen for notifications.

One-liner to install and get it running (Requires administrator privileges):

```
Start-Process powershell.exe -Verb runas -ArgumentList '-Command iex (iwr https://github.com/bluskript/xsoverlay-notifier/releases/download/latest/install.ps1).content'
```

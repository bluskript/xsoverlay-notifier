Import-Certificate -FilePath .\code_signing.crt -Cert Cert:\CurrentUser\TrustedPublisher
Add-AppPackage -Path ./out/XSOverlayNotifier.msix
Start-Process "shell:AppsFolder\$((Get-StartApps | Where-Object {$_.Name -eq 'XSOverlay Notifier'}).'AppID')"

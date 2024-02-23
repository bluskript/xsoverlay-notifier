#Requires -RunAsAdministrator
$Installer = New-TemporaryFile
Invoke-WebRequest -Uri https://github.com/bluskript/xsoverlay-notifier/releases/download/latest/XSOverlayNotifier.msix -OutFile $Installer
# Import-Certificate -FilePath $Cert -Cert Cert:\CurrentUser\Root
Add-AppPackage -AllowUnsigned -Path $Installer
Start-Process "shell:AppsFolder\$((Get-StartApps | Where-Object {$_.Name -eq 'XSOverlay Notifier'}).'AppID')"

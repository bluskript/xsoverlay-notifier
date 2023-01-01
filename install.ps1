#Requires -RunAsAdministrator
$Cert = New-TemporaryFile
$Installer = New-TemporaryFile
Invoke-WebRequest -Uri https://github.com/bluskript/xsoverlay-notifier/releases/download/latest/code_signing.crt -OutFile $Cert
Invoke-WebRequest -Uri https://github.com/bluskript/xsoverlay-notifier/releases/download/latest/XSOverlayNotifier.msix -OutFile $Installer
Import-Certificate -FilePath $Cert -Cert Cert:\LocalMachine\TrustedPeople
# Import-Certificate -FilePath $Cert -Cert Cert:\CurrentUser\Root
Add-AppPackage -Path $Installer
Start-Process "shell:AppsFolder\$((Get-StartApps | Where-Object {$_.Name -eq 'XSOverlay Notifier'}).'AppID')"

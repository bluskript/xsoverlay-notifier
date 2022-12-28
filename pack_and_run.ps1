./pack.ps1
Set-Location ..
Remove-AppPackage (Get-AppPackage -name 'Blusk.XSOverlayNotifier').'PackageFullName'
Add-AppPackage -Path ./out/XSOverlayNotifier.msix
Start-Process "shell:AppsFolder\$((Get-StartApps | Where-Object {$_.Name -eq 'XSOverlay Notifier'}).'AppID')"
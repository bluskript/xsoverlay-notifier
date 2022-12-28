cargo build
Copy-Item target/debug/xsoverlay_notifier.exe ./windows-packaging
Set-Location windows-packaging
MakeAppx.exe pack /d . /p ../out/XSOverlayNotifier.msix /nv /o
SignTool.exe sign /a /v /fd SHA256 /f ../../common/mycert.pfx /p qwertyuiop ../out/XSOverlayNotifier.msix

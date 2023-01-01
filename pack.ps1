Copy-Item target/debug/xsoverlay_notifier.exe ./windows-packaging
Set-Location windows-packaging
& "C:\Program Files (x86)\Windows Kits\10\bin\10.0.22000.0\x64\MakeAppx.exe" pack /d . /p ../out/XSOverlayNotifier.msix /nv /o
& "C:\Program Files (x86)\Windows Kits\10\bin\10.0.22000.0\x64\SignTool.exe" sign /a /v /fd SHA256 /f ../../common/mycert.pfx /p qwertyuiop ../out/XSOverlayNotifier.msix

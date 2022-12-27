cargo.exe build
cp target/debug/xsoverlay_notifier.exe ./windows-packaging
cd windows-packaging
MakeAppx.exe pack /d . /p ../out/XSOverlayNotifier.msix /nv
SignTool.exe sign /a /v /fd SHA256 /f ../../common/mycert.pfx /p qwertyuiop ../out/XSOverlayNotifier.msix
$bytes = [Convert]::FromBase64String($env:SIGN_CERT)
[Io.File]::WriteAllBytes('./sign_cert.pfx', $bytes)
$SIGN_CERT_PASSWORD_SECURE = ConvertTo-SecureString -String $env:SIGN_CERT_PASSWORD -AsPlainText -Force
Import-PfxCertificate -Password $SIGN_CERT_PASSWORD_SECURE -FilePath ./sign_cert.pfx -Cert Cert:\CurrentUser\TrustedPublisher
Export-Certificate -Cert (Get-ChildItem Cert:\CurrentUser\TrustedPublisher -CodeSigningCert)[0] -FilePath code_signing.crt
Copy-Item target/release/xsoverlay_notifier.exe ./windows-packaging
Set-Location windows-packaging
& "C:\Program Files (x86)\Windows Kits\10\bin\10.0.22000.0\x64\MakeAppx.exe" pack /d . /p ../out/XSOverlayNotifier.msix /nv /o
& "C:\Program Files (x86)\Windows Kits\10\bin\10.0.22000.0\x64\SignTool.exe" sign /a /v /fd SHA256 /f ../sign_cert.pfx /p $env:SIGN_CERT_PASSWORD ../out/XSOverlayNotifier.msix

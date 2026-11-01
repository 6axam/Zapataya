param([Parameter(Mandatory = $true)][string]$File)
$ErrorActionPreference = 'Stop'

# Signing is opt-in. When enabled it fails closed: a release cannot silently
# fall back to an unsigned EXE if the certificate is missing or invalid.
if ($env:WINDOWS_SIGNING_ENABLED -ne 'true') {
    Write-Warning 'Windows signing is not configured; the EXE is unsigned.'
    return
}
if (-not $env:WINDOWS_CERTIFICATE_PFX_BASE64 -or -not $env:WINDOWS_CERTIFICATE_PASSWORD) {
    throw 'Windows signing is enabled but the certificate or password is missing.'
}

$sdkBin = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits\10\bin'
$signTool = Get-ChildItem $sdkBin -Directory | Sort-Object Name -Descending |
    ForEach-Object { Join-Path $_.FullName 'x64\signtool.exe' } |
    Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
if (-not $signTool) { throw 'Windows SDK signtool.exe was not found.' }

$pfxPath = Join-Path $env:RUNNER_TEMP ([guid]::NewGuid().ToString() + '.pfx')
$cert = $null
try {
    [IO.File]::WriteAllBytes($pfxPath, [Convert]::FromBase64String($env:WINDOWS_CERTIFICATE_PFX_BASE64))
    $password = ConvertTo-SecureString $env:WINDOWS_CERTIFICATE_PASSWORD -AsPlainText -Force
    $cert = Import-PfxCertificate -FilePath $pfxPath -CertStoreLocation Cert:\CurrentUser\My -Password $password
    if (@($cert).Count -ne 1 -or -not $cert.HasPrivateKey) { throw 'Expected exactly one certificate with a private key.' }
    & $signTool sign /fd SHA256 /sha1 $cert.Thumbprint /tr 'http://timestamp.digicert.com' /td SHA256 $File
    if ($LASTEXITCODE -ne 0) { throw "SignTool signing failed: $LASTEXITCODE" }
    & $signTool verify /pa /all /v $File
    if ($LASTEXITCODE -ne 0) { throw "Windows did not trust the signature: $LASTEXITCODE" }
    $signature = Get-AuthenticodeSignature -FilePath $File
    if ($signature.Status -ne 'Valid') { throw "Signature is not trusted: $($signature.Status)" }
    Write-Host 'Windows signature and certificate chain verified.'
}
finally {
    if ($cert) { $cert | Remove-Item -ErrorAction SilentlyContinue }
    Remove-Item -LiteralPath $pfxPath -Force -ErrorAction SilentlyContinue
}

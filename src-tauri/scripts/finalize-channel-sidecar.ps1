# Move channel-sidecar-*.exe.new -> binaries/ after quitting the running app.
$ErrorActionPreference = "Stop"
$Root = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$BinDir = Join-Path $Root "src-tauri\binaries"
$Target = if ($env:CARGO_BUILD_TARGET) { $env:CARGO_BUILD_TARGET } else { "x86_64-pc-windows-msvc" }
$OutPath = Join-Path $BinDir "channel-sidecar-$Target.exe"
$Staging = "$OutPath.new"

if (-not (Test-Path -LiteralPath $Staging)) {
    Write-Host "Nothing to finalize: $Staging does not exist."
    exit 0
}

try {
    Move-Item -LiteralPath $Staging -Destination $OutPath -Force
    Write-Host "Installed $OutPath"
} catch [System.IO.IOException] {
    Write-Error "Still locked: $OutPath — quit SupportFlow and retry."
}

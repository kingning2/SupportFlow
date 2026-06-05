# Build license-verifier exe from subscription-activation-generator into src-tauri/binaries/
param(
    [string]$Target = ""
)

$ErrorActionPreference = "Stop"
$Root = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$ToolDir = Join-Path $Root "subscription-activation-generator"
$BinDir = Join-Path $Root "src-tauri\binaries"

if (-not $Target) {
    $Target = & rustup show active-toolchain 2>$null | Select-String -Pattern "host: (\S+)" | ForEach-Object { $_.Matches.Groups[1].Value }
    if (-not $Target) {
        $Target = "x86_64-pc-windows-msvc"
    }
}

New-Item -ItemType Directory -Force -Path $BinDir | Out-Null

Push-Location $ToolDir
try {
    Write-Host "Building license-verifier for $Target ..."
    cargo build --release --bin license-verifier
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
finally {
    Pop-Location
}

$Src = Join-Path $ToolDir "target\release\license-verifier.exe"
$Dst = Join-Path $BinDir "license-verifier-$Target.exe"
Copy-Item -Force $Src $Dst
Write-Host "Copied -> $Dst"

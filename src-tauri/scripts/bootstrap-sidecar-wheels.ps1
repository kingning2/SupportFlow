# Download cp310 Windows wheels with a working pip (default: python 3.12).
# ntwork is not on PyPI; we fetch the cp310 wheel from ntwork-bin-backup on GitHub.
$ErrorActionPreference = "Stop"
$Root = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$WheelDir = Join-Path $Root "src-tauri\channel_agent\_wheels\sidecar-pip"
$BootstrapPy = if ($env:CHANNEL_BOOTSTRAP_PYTHON) { $env:CHANNEL_BOOTSTRAP_PYTHON } else { (Get-Command python -ErrorAction Stop).Source }

New-Item -ItemType Directory -Force -Path $WheelDir | Out-Null
Write-Host "Bootstrap pip: $BootstrapPy"
Write-Host "Wheel dir: $WheelDir"

$pyVer = "310"
$plat = "win_amd64"
$onlyBin = @("--python-version", $pyVer, "--platform", $plat, "--only-binary=:all:")

Write-Host "Downloading PyInstaller + deps..."
& $BootstrapPy -m pip download pyinstaller importlib-metadata zipp -d $WheelDir @onlyBin

$sidecarReq = Join-Path $Root "src-tauri\channel_agent\requirements-sidecar.txt"
if (Test-Path $sidecarReq) {
    Write-Host "Downloading channel sidecar runtime deps..."
    & $BootstrapPy -m pip download -r $sidecarReq -d $WheelDir `
        --python-version 310 --platform win_amd64 --only-binary=:all:
    if ($LASTEXITCODE -ne 0) { throw "pip download requirements-sidecar.txt failed" }
    foreach ($dep in @("async-timeout", "exceptiongroup", "typing-extensions")) {
        & $BootstrapPy -m pip download $dep -d $WheelDir --only-binary=:all: | Out-Null
    }
}

Write-Host "Downloading pilk..."
& $BootstrapPy -m pip download pilk -d $WheelDir @onlyBin

. (Join-Path $PSScriptRoot "sync-ntwork-wheels.ps1")
Sync-NtworkWheels -WheelDir $WheelDir -BootstrapPy $BootstrapPy

Write-Host "Done. Wheels in $WheelDir"

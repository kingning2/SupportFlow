# Build channel sidecar exe (PyInstaller one-file) into src-tauri/binaries/
# Wework (ntwork): Python 3.10 — set CHANNEL_SIDECAR_PYTHON or rely on `py -3.10`.
# If py 3.10 pip cannot reach PyPI, run: pnpm run bootstrap:sidecar-wheels
param(
    [switch]$DepsOnly
)
$ErrorActionPreference = "Stop"
$Root = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$ChannelAgent = Join-Path $Root "src-tauri\channel_agent"
$BinDir = Join-Path $Root "src-tauri\binaries"
$WheelDir = Join-Path $ChannelAgent "_wheels\sidecar-pip"
$Target = if ($env:CARGO_BUILD_TARGET) { $env:CARGO_BUILD_TARGET } else { "x86_64-pc-windows-msvc" }
$OutName = "channel-sidecar-$Target.exe"
$OutPath = Join-Path $BinDir $OutName

function Resolve-SidecarPython {
    if ($env:CHANNEL_SIDECAR_PYTHON -and (Test-Path $env:CHANNEL_SIDECAR_PYTHON)) {
        return $env:CHANNEL_SIDECAR_PYTHON
    }
    try {
        $exe = & py -3.10 -c "import sys; print(sys.executable)" 2>$null
        if ($exe -and (Test-Path $exe)) {
            Write-Host "Using Python 3.10 for sidecar (wework/ntwork): $exe"
            return $exe
        }
    } catch { }
    $fallback = (Get-Command python -ErrorAction Stop).Source
    Write-Host "Using default python (ntwork requires Python 3.10): $fallback"
    return $fallback
}

function Invoke-Quiet {
    param([scriptblock]$Command)
    $prev = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        & $Command *>&1 | Out-Null
        if ($null -ne $LASTEXITCODE) { return $LASTEXITCODE }
        return 0
    } finally {
        $ErrorActionPreference = $prev
    }
}

function Resolve-BootstrapPython {
    if ($env:CHANNEL_BOOTSTRAP_PYTHON -and (Test-Path $env:CHANNEL_BOOTSTRAP_PYTHON)) {
        return $env:CHANNEL_BOOTSTRAP_PYTHON
    }
    return (Get-Command python -ErrorAction Stop).Source
}

function Ensure-WheelDir {
    if (-not (Test-Path $WheelDir)) {
        New-Item -ItemType Directory -Force -Path $WheelDir | Out-Null
    }
}

function Sync-PyInstallerWheels {
    param([string]$BootstrapPy)
    Ensure-WheelDir
    $need = -not (Get-ChildItem -Path $WheelDir -Filter "pyinstaller-*.whl" -ErrorAction SilentlyContinue)
    if ($need) {
        Write-Host "Downloading PyInstaller wheels (via $BootstrapPy)..."
        & $BootstrapPy -m pip download pyinstaller importlib-metadata zipp -d $WheelDir `
            --python-version 310 --platform win_amd64 --only-binary=:all:
        if ($LASTEXITCODE -ne 0) { throw "pip download PyInstaller failed" }
    }
}

function Install-FromWheelDir {
    param([string]$PythonExe, [string[]]$ExtraArgs = @())
    $pipArgs = @("-m", "pip", "install", "--no-index", "--find-links", $WheelDir) + $ExtraArgs
    $code = Invoke-Quiet { & $PythonExe @pipArgs }
    return ($code -eq 0)
}

function Ensure-PyInstaller {
    param([string]$PythonExe, [string]$BootstrapPy)
    if ((Invoke-Quiet { & $PythonExe -m PyInstaller --version }) -eq 0) { return }

    Write-Host "Installing PyInstaller into sidecar Python..."
    $savedProxy = $env:HTTP_PROXY, $env:HTTPS_PROXY, $env:ALL_PROXY
    $env:HTTP_PROXY = $null; $env:HTTPS_PROXY = $null; $env:ALL_PROXY = $null
    $pipCode = Invoke-Quiet { & $PythonExe -m pip install pyinstaller --disable-pip-version-check }
    $env:HTTP_PROXY = $savedProxy[0]; $env:HTTPS_PROXY = $savedProxy[1]; $env:ALL_PROXY = $savedProxy[2]

    if ($pipCode -ne 0) {
        Write-Host "Online pip failed; using offline wheels..."
        Sync-PyInstallerWheels -BootstrapPy $BootstrapPy
        if (-not (Install-FromWheelDir -PythonExe $PythonExe -ExtraArgs @("pyinstaller"))) {
            throw "Failed to install PyInstaller for $PythonExe (run: pnpm run bootstrap:sidecar-wheels)"
        }
    }

    if ((Invoke-Quiet { & $PythonExe -m PyInstaller --version }) -ne 0) {
        throw "PyInstaller not available on $PythonExe"
    }
}

function Sync-PilkWheel {
    param([string]$BootstrapPy)
    Ensure-WheelDir
    if (-not (Get-ChildItem -Path $WheelDir -Filter "pilk*.whl" -ErrorAction SilentlyContinue)) {
        Write-Host "Downloading pilk wheel..."
        & $BootstrapPy -m pip download pilk -d $WheelDir `
            --python-version 310 --platform win_amd64 --only-binary=:all:
        if ($LASTEXITCODE -ne 0) { throw "pip download pilk failed" }
    }
}

function Test-NtworkInstalled {
    param([string]$PythonExe)
    $code = 1
    try {
        $code = & cmd /c "`"$PythonExe`" -c `"import ntwork`" 2>nul"
        if ($null -eq $code) { $code = $LASTEXITCODE }
    } catch { }
    return ($code -eq 0)
}

function Test-SidecarDepsInstalled {
    param([string]$PythonExe)
    $code = 1
    try {
        $code = & cmd /c "`"$PythonExe`" -c `"import requests, PIL`" 2>nul"
        if ($null -eq $code) { $code = $LASTEXITCODE }
    } catch { }
    return ($code -eq 0)
}

function Sync-SidecarWheels {
    param([string]$BootstrapPy)
    $req = Join-Path $ChannelAgent "requirements-sidecar.txt"
    if (-not (Test-Path $req)) { return }
    Ensure-WheelDir
    Write-Host "Downloading sidecar dependency wheels..."
    & $BootstrapPy -m pip download -r $req -d $WheelDir `
        --python-version 310 --platform win_amd64 --only-binary=:all:
    if ($LASTEXITCODE -ne 0) { throw "pip download requirements-sidecar.txt failed" }
    foreach ($dep in @("async-timeout", "exceptiongroup", "typing-extensions")) {
        Invoke-Quiet { & $BootstrapPy -m pip download $dep -d $WheelDir --only-binary=:all: } | Out-Null
    }
}

function Install-SidecarDeps {
    param([string]$PythonExe, [string]$BootstrapPy)

    if (Test-SidecarDepsInstalled -PythonExe $PythonExe) {
        Write-Host "Sidecar channel deps already installed"
        return
    }

    $req = Join-Path $ChannelAgent "requirements-sidecar.txt"
    if (-not (Test-Path $req)) {
        Write-Warning "Missing requirements-sidecar.txt"
        return
    }

    Write-Host "Installing sidecar channel dependencies..."
    $savedProxy = $env:HTTP_PROXY, $env:HTTPS_PROXY, $env:ALL_PROXY
    $env:HTTP_PROXY = $null; $env:HTTPS_PROXY = $null; $env:ALL_PROXY = $null
    $pipCode = Invoke-Quiet { & $PythonExe -m pip install -r $req --disable-pip-version-check }
    $env:HTTP_PROXY = $savedProxy[0]; $env:HTTPS_PROXY = $savedProxy[1]; $env:ALL_PROXY = $savedProxy[2]

    if ($pipCode -ne 0) {
        Write-Host "Online pip failed; installing sidecar deps from offline wheels..."
        Sync-SidecarWheels -BootstrapPy $BootstrapPy
        if (-not (Install-FromWheelDir -PythonExe $PythonExe -ExtraArgs @("-r", $req))) {
            throw "Failed to install sidecar deps for $PythonExe (run: pnpm run bootstrap:sidecar-wheels)"
        }
    }

    if (-not (Test-SidecarDepsInstalled -PythonExe $PythonExe)) {
        throw "Sidecar deps incomplete after install (requests/Pillow)"
    }
}

function Install-WeworkDeps {
    param([string]$PythonExe, [string]$BootstrapPy)

    if ($env:CHANNEL_SKIP_WEWORK_DEPS -eq "1") {
        Write-Host "CHANNEL_SKIP_WEWORK_DEPS=1 — skipping wework deps"
        return $false
    }

    if (Test-NtworkInstalled -PythonExe $PythonExe) {
        Write-Host "ntwork already installed on sidecar Python"
        return $true
    }

    $weworkReq = Join-Path $ChannelAgent "requirements-wework.txt"
    Write-Host "Installing wework deps (pilk; ntwork via wheel if present)..."

    $savedProxy = $env:HTTP_PROXY, $env:HTTPS_PROXY, $env:ALL_PROXY
    $env:HTTP_PROXY = $null; $env:HTTPS_PROXY = $null; $env:ALL_PROXY = $null
    if (Test-Path $weworkReq) {
        Invoke-Quiet { & $PythonExe -m pip install -r $weworkReq --disable-pip-version-check } | Out-Null
    }
    $env:HTTP_PROXY = $savedProxy[0]; $env:HTTPS_PROXY = $savedProxy[1]; $env:ALL_PROXY = $savedProxy[2]

    if (-not (Test-NtworkInstalled -PythonExe $PythonExe)) {
        Sync-PilkWheel -BootstrapPy $BootstrapPy
        Install-FromWheelDir -PythonExe $PythonExe -ExtraArgs @("pilk") | Out-Null

        . (Join-Path $PSScriptRoot "sync-ntwork-wheels.ps1")
        Sync-NtworkWheels -WheelDir $WheelDir -BootstrapPy $BootstrapPy

        if ($env:CHANNEL_NTWORK_WHEEL -and (Test-Path $env:CHANNEL_NTWORK_WHEEL)) {
            Write-Host "Installing ntwork from CHANNEL_NTWORK_WHEEL..."
            Install-FromWheelDir -PythonExe $PythonExe -ExtraArgs @($env:CHANNEL_NTWORK_WHEEL) | Out-Null
        } else {
            Write-Host "Installing ntwork + pyee + xcgui from offline wheels..."
            Install-FromWheelDir -PythonExe $PythonExe -ExtraArgs @("ntwork", "pyee", "xcgui") | Out-Null
        }
    }

    if (Test-NtworkInstalled -PythonExe $PythonExe) {
        Write-Host "wework deps OK (ntwork + pilk)"
        return $true
    }

    Write-Warning @"
ntwork is not installed — wework channel will not work in the built exe.
  - Run: pnpm run bootstrap:sidecar-wheels
  - Place ntwork-*-cp310-*-win_amd64.whl in: $WheelDir
  - Or set CHANNEL_NTWORK_WHEEL to the .whl path
  - Or set CHANNEL_SKIP_WEWORK_DEPS=1 to silence this warning
"@
    return $false
}

function Get-SitePackageDir {
    param([string]$PythonExe, [string]$PackageName)
    $out = & $PythonExe -c "import $PackageName as m, os; print(os.path.dirname(m.__file__))" 2>$null
    if ($LASTEXITCODE -ne 0 -or -not $out) {
        throw "Cannot locate site-packages for $PackageName"
    }
    return $out.Trim()
}

function Publish-SidecarExe {
    param([string]$Built, [string]$OutPath)

    $staging = "$OutPath.new"
    $maxAttempts = 6

    for ($i = 1; $i -le $maxAttempts; $i++) {
        try {
            Copy-Item -LiteralPath $Built -Destination $OutPath -Force -ErrorAction Stop
            if (Test-Path -LiteralPath $staging) {
                Remove-Item -LiteralPath $staging -Force -ErrorAction SilentlyContinue
            }
            Write-Host "Wrote $OutPath"
            return
        } catch [System.IO.IOException] {
            if ($i -lt $maxAttempts) {
                Write-Host "Target locked ($OutPath), retry $i/$maxAttempts in 2s — quit SupportFlow if it is running..."
                Start-Sleep -Seconds 2
            }
        }
    }

    Copy-Item -LiteralPath $Built -Destination $staging -Force
    throw @"
PyInstaller build OK, but the installed sidecar could not be replaced (file in use):
  $OutPath

Fresh binary written to:
  $staging

1. Quit SupportFlow (and end any channel-sidecar-*.exe in Task Manager)
2. Then run ONE of:
     Move-Item -LiteralPath '$staging' -Destination '$OutPath' -Force
     pnpm run finalize:channel-sidecar
     pnpm run build:channel-sidecar
"@
}

$PythonExe = Resolve-SidecarPython
$BootstrapPy = Resolve-BootstrapPython

if ($DepsOnly) {
    Install-SidecarDeps -PythonExe $PythonExe -BootstrapPy $BootstrapPy
    $hasNtwork = Install-WeworkDeps -PythonExe $PythonExe -BootstrapPy $BootstrapPy
    Write-Host ""
    Write-Host "Dev sidecar Python: $PythonExe"
    if (-not $hasNtwork) {
        Write-Warning "ntwork not installed — wework channel will fail until wheels install succeeds."
        exit 1
    }
    Write-Host @"

Add to project root .env:
  CHANNEL_PYTHON_EXECUTABLE=$PythonExe

Or set user env CHANNEL_PYTHON_EXECUTABLE to the path above.
Then restart: pnpm run tauri dev
"@
    exit 0
}

Ensure-PyInstaller -PythonExe $PythonExe -BootstrapPy $BootstrapPy
Install-SidecarDeps -PythonExe $PythonExe -BootstrapPy $BootstrapPy
$hasNtwork = Install-WeworkDeps -PythonExe $PythonExe -BootstrapPy $BootstrapPy

New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
Push-Location $ChannelAgent

$hidden = @(
    "channel",
    "bridge",
    "common",
    "config",
    "voice",
    "lib",
    "pilk",
    "requests",
    "urllib3",
    "certifi",
    "charset_normalizer",
    "idna",
    "PIL",
    "qrcode",
    "yaml",
    "dotenv",
    "web",
    "websocket",
    "Crypto",
)
if ($hasNtwork) {
    $hidden += @("ntwork", "ntwork.core", "ntwork.wc", "ntwork.conf", "ntwork.const", "pyee")
}

$entryScript = Join-Path $ChannelAgent "channel\__main__.py"
$pyArgs = @(
    "--noconfirm",
    "--clean",
    "--onefile",
    "--console",
    "--paths", $ChannelAgent,
    "--name", "channel-sidecar-build",
    $entryScript
)
foreach ($m in $hidden) {
    $pyArgs += "--hidden-import"
    $pyArgs += $m
}
$pyArgs += "--collect-submodules"
$pyArgs += "channel"
$pyArgs += "--collect-submodules"
$pyArgs += "lib"
if ($hasNtwork) {
    # Bundle ntwork/pilk/xcgui into the one-file sidecar (no runtime pip / site-packages).
    $ntworkDir = Get-SitePackageDir -PythonExe $PythonExe -PackageName "ntwork"
    $xcguiDir = Get-SitePackageDir -PythonExe $PythonExe -PackageName "xcgui"
    $wcProbe = Join-Path $ntworkDir "wc\wcprobe.cp310-win_amd64.pyd"
    $xcguiPyd = Join-Path $xcguiDir "_xcgui.cp310-win_amd64.pyd"
    $pyArgs += "--exclude-module"
    $pyArgs += "xcgui"
    $pyArgs += "--add-data"
    $pyArgs += "$ntworkDir;ntwork"
    $pyArgs += "--add-data"
    $pyArgs += "$xcguiDir;xcgui"
    $pyArgs += "--add-binary"
    $pyArgs += "$wcProbe;ntwork\wc"
    $pyArgs += "--add-binary"
    $pyArgs += "$xcguiPyd;xcgui"
}

Write-Host "PyInstaller: $($pyArgs -join ' ')"
& $PythonExe -m PyInstaller @pyArgs
if ($LASTEXITCODE -ne 0) {
    Pop-Location
    throw "PyInstaller failed (exit $LASTEXITCODE)"
}

$built = Join-Path $ChannelAgent "dist\channel-sidecar-build.exe"
if (-not (Test-Path $built)) {
    Pop-Location
    throw "PyInstaller output not found: $built"
}

Publish-SidecarExe -Built $built -OutPath $OutPath
Pop-Location

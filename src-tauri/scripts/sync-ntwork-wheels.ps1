# Shared: download ntwork + pyee + xcgui wheels for Python 3.10 / wework channel.
$script:NtworkWhlName = "ntwork-0.1.3-cp310-cp310-win_amd64.whl"
$script:NtworkWhlUrl = "https://github.com/hanfangyuan4396/ntwork-bin-backup/raw/main/ntwork-whl/$($script:NtworkWhlName)"

function Sync-NtworkWheels {
    param(
        [string]$WheelDir,
        [string]$BootstrapPy
    )

    New-Item -ItemType Directory -Force -Path $WheelDir | Out-Null
    $ntworkPath = Join-Path $WheelDir $script:NtworkWhlName

    if (-not (Test-Path -LiteralPath $ntworkPath)) {
        Write-Host "Downloading $($script:NtworkWhlName) ..."
        Invoke-WebRequest -Uri $script:NtworkWhlUrl -OutFile $ntworkPath -UseBasicParsing
    } else {
        Write-Host "Found ntwork wheel: $($script:NtworkWhlName)"
    }

    $onlyBin = @("--python-version", "310", "--platform", "win_amd64", "--only-binary=:all:")
    foreach ($dep in @("pyee", "xcgui")) {
        if (-not (Get-ChildItem -Path $WheelDir -Filter "$dep*.whl" -ErrorAction SilentlyContinue)) {
            Write-Host "Downloading $dep for ntwork..."
            & $BootstrapPy -m pip download $dep -d $WheelDir @onlyBin
            if ($LASTEXITCODE -ne 0) { throw "pip download $dep failed" }
        }
    }
}

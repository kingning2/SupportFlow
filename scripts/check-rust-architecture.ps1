$ErrorActionPreference = "Stop"

function Assert-NoMatches {
  param(
    [string]$Pattern,
    [string[]]$Paths,
    [string]$Message
  )

  $results = & rg -n --glob "*.rs" -e $Pattern @Paths 2>$null
  if ($LASTEXITCODE -eq 0 -and $results) {
    Write-Host $results
    throw $Message
  }
}

function Assert-NoLiteralMatches {
  param(
    [string]$Pattern,
    [string[]]$Paths,
    [string]$Message
  )

  $results = & rg -n --glob "*.rs" -F -e $Pattern @Paths 2>$null
  if ($LASTEXITCODE -eq 0 -and $results) {
    Write-Host $results
    throw $Message
  }
}

function Assert-PathMissing {
  param(
    [string]$Path,
    [string]$Message
  )

  if (Test-Path $Path) {
    throw $Message
  }
}

$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

Assert-PathMissing "src-tauri/src/utils/skills_config.rs" "skills_config.rs must stay out of src-tauri/src/utils"
Assert-PathMissing "src-tauri/src/utils/skills_installer.rs" "skills_installer.rs must stay out of src-tauri/src/utils"
Assert-PathMissing "src-tauri/src/context/channel/config.rs" "channel config orchestration must stay out of src-tauri/src/context/channel"

Assert-NoMatches "crate::agent::|crate::bridge::" @("src-tauri/src") "Use services::agent / services::bridge directly inside src-tauri/src"
Assert-NoMatches "Command::new\(" @("src-tauri/src/cmd") "cmd layer must not spawn processes directly"
Assert-NoMatches "reqwest::|walkdir::|std::fs::|fs::read_dir" @("src-tauri/src/cmd") "cmd layer must remain a thin entry layer"
Assert-NoLiteralMatches 'Command::new("python"' @("src-tauri/src/context", "src-tauri/src/services") "Python process spawning must stay behind src-tauri/src/python"
Assert-NoLiteralMatches "Command::new('python'" @("src-tauri/src/context", "src-tauri/src/services") "Python process spawning must stay behind src-tauri/src/python"

Write-Host "Rust architecture checks passed."

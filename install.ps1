#Requires -Version 5.1
<#
.SYNOPSIS
    c9watch installer for Windows

.DESCRIPTION
    Downloads and installs the latest c9watch release for Windows.

.EXAMPLE
    # Run directly from PowerShell:
    irm https://raw.githubusercontent.com/minchenlee/c9watch/main/install.ps1 | iex

    # Or download and run:
    .\install.ps1
#>

$ErrorActionPreference = 'Stop'

$REPO = "minchenlee/c9watch"
$APP_NAME = "c9watch"

function Write-Info { param([string]$Message) Write-Host "=> $Message" -ForegroundColor Cyan }
function Write-Err  { param([string]$Message) Write-Host "Error: $Message" -ForegroundColor Red; exit 1 }

# --- Check OS ---
if ($env:OS -ne 'Windows_NT') {
    Write-Err "This installer is for Windows only. Detected OS: $($env:OS)"
}

$osVersion = [System.Environment]::OSVersion.Version
if ($osVersion.Build -lt 17134) {
    Write-Err "c9watch requires Windows 10 (1803) or later. Current build: $($osVersion.Build)"
}

# --- Only x64 supported ---
$arch = $env:PROCESSOR_ARCHITECTURE
if ($arch -ne 'AMD64') {
    Write-Err "c9watch currently only supports x64 architecture. Detected: $arch"
}

Write-Info "Detected Windows x64 (build $($osVersion.Build))"

# --- Find latest release ---
Write-Info "Fetching latest release..."

try {
    $releaseInfo = Invoke-RestMethod -Uri "https://api.github.com/repos/$REPO/releases/latest" -UseBasicParsing
    $latestTag = $releaseInfo.tag_name
} catch {
    Write-Err "Could not determine the latest release. Check https://github.com/$REPO/releases"
}

if (-not $latestTag) {
    Write-Err "Could not determine the latest release tag."
}

$version = $latestTag.TrimStart('v')
Write-Info "Latest version: $latestTag"

# --- Look for NSIS installer first, then MSI ---
$nsisPattern = "${APP_NAME}_${version}_x64-setup.exe"
$msiPattern  = "${APP_NAME}_${version}_x64_en-US.msi"

$downloadAsset = $null
$installerType = $null

foreach ($asset in $releaseInfo.assets) {
    if ($asset.name -eq $nsisPattern) {
        $downloadAsset = $asset
        $installerType = 'nsis'
        break
    }
    if ($asset.name -eq $msiPattern) {
        $downloadAsset = $asset
        $installerType = 'msi'
    }
}

if (-not $downloadAsset) {
    Write-Err "No installer found for x64 in release $latestTag.`nLooked for: $nsisPattern or $msiPattern"
}

$downloadUrl = $downloadAsset.browser_download_url
$fileName = $downloadAsset.name

Write-Info "Downloading $fileName..."

# --- Download ---
$tempDir = Join-Path $env:TEMP "c9watch-install-$(Get-Random)"
New-Item -ItemType Directory -Path $tempDir -Force | Out-Null
$installerPath = Join-Path $tempDir $fileName

try {
    $ProgressPreference = 'SilentlyContinue'
    Invoke-WebRequest -Uri $downloadUrl -OutFile $installerPath -UseBasicParsing
    $ProgressPreference = 'Continue'
} catch {
    Write-Err "Failed to download installer: $_"
}

Write-Info "Downloaded to $installerPath"

# --- Install ---
Write-Info "Running installer..."

try {
    if ($installerType -eq 'nsis') {
        $process = Start-Process -FilePath $installerPath -Wait -PassThru
    } else {
        $process = Start-Process -FilePath 'msiexec.exe' -ArgumentList "/i `"$installerPath`"" -Wait -PassThru
    }
    if ($process.ExitCode -ne 0) {
        Write-Err "Installer exited with code $($process.ExitCode)"
    }
} catch {
    Write-Err "Installation failed: $_"
}

# --- Cleanup ---
Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue

# --- Done ---
Write-Host ""
Write-Info "$APP_NAME has been installed successfully!"
Write-Info "You can launch it from the Start Menu or by searching for '$APP_NAME'."
Write-Host ""

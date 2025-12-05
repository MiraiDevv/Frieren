# Frieren Downloader - Setup Script
# This script installs all required dependencies for running the application

Write-Host "============================================" -ForegroundColor Cyan
Write-Host "   Frieren Downloader - Dependency Setup   " -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""

# Check if running as Administrator for winget
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

# Function to check if a command exists
function Test-CommandExists {
    param([string]$Command)
    $null -ne (Get-Command $Command -ErrorAction SilentlyContinue)
}

# Function to refresh PATH in current session
function Update-Path {
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
}

Write-Host "[1/5] Checking for winget..." -ForegroundColor Yellow
if (-not (Test-CommandExists "winget")) {
    Write-Host "ERROR: winget is not installed. Please install App Installer from Microsoft Store." -ForegroundColor Red
    exit 1
}
Write-Host "  -> winget found!" -ForegroundColor Green

# Install Rust
Write-Host ""
Write-Host "[2/5] Checking for Rust..." -ForegroundColor Yellow
if (-not (Test-CommandExists "cargo")) {
    Write-Host "  -> Installing Rust via rustup..." -ForegroundColor Cyan
    winget install Rustlang.Rustup -e --silent --accept-package-agreements --accept-source-agreements
    Update-Path
    if (Test-CommandExists "cargo") {
        Write-Host "  -> Rust installed successfully!" -ForegroundColor Green
    }
    else {
        Write-Host "  -> Rust installed. Please restart terminal and run this script again." -ForegroundColor Yellow
    }
}
else {
    $rustVersion = cargo --version
    Write-Host "  -> Rust already installed: $rustVersion" -ForegroundColor Green
}

# Install Node.js
Write-Host ""
Write-Host "[3/5] Checking for Node.js..." -ForegroundColor Yellow
if (-not (Test-CommandExists "node")) {
    Write-Host "  -> Installing Node.js LTS..." -ForegroundColor Cyan
    winget install OpenJS.NodeJS.LTS -e --silent --accept-package-agreements --accept-source-agreements
    Update-Path
    if (Test-CommandExists "node") {
        Write-Host "  -> Node.js installed successfully!" -ForegroundColor Green
    }
    else {
        Write-Host "  -> Node.js installed. Please restart terminal and run this script again." -ForegroundColor Yellow
    }
}
else {
    $nodeVersion = node --version
    Write-Host "  -> Node.js already installed: $nodeVersion" -ForegroundColor Green
}

# Install Bun
Write-Host ""
Write-Host "[4/5] Checking for Bun..." -ForegroundColor Yellow
if (-not (Test-CommandExists "bun")) {
    Write-Host "  -> Installing Bun..." -ForegroundColor Cyan
    winget install Oven-sh.Bun -e --silent --accept-package-agreements --accept-source-agreements
    Update-Path
    if (Test-CommandExists "bun") {
        Write-Host "  -> Bun installed successfully!" -ForegroundColor Green
    }
    else {
        Write-Host "  -> Bun installed. Please restart terminal and run this script again." -ForegroundColor Yellow
    }
}
else {
    $bunVersion = bun --version
    Write-Host "  -> Bun already installed: $bunVersion" -ForegroundColor Green
}

# Install FFmpeg
Write-Host ""
Write-Host "[5/5] Checking for FFmpeg..." -ForegroundColor Yellow
if (-not (Test-CommandExists "ffmpeg")) {
    Write-Host "  -> Installing FFmpeg..." -ForegroundColor Cyan
    winget install Gyan.FFmpeg -e --silent --accept-package-agreements --accept-source-agreements
    Update-Path
    if (Test-CommandExists "ffmpeg") {
        Write-Host "  -> FFmpeg installed successfully!" -ForegroundColor Green
    }
    else {
        Write-Host "  -> FFmpeg installed. Please restart terminal and run this script again." -ForegroundColor Yellow
    }
}
else {
    $ffmpegVersion = (ffmpeg -version | Select-Object -First 1)
    Write-Host "  -> FFmpeg already installed: $ffmpegVersion" -ForegroundColor Green
}

Write-Host ""
Write-Host "============================================" -ForegroundColor Cyan
Write-Host "          Installation Complete!           " -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Restarting terminal to apply changes..." -ForegroundColor Yellow
Start-Sleep -Seconds 2

# Get the current directory for the new shell
$currentDir = Get-Location

# Start a new PowerShell window in the same directory with next steps
$startupCommand = @"
Set-Location '$currentDir'
Write-Host ''
Write-Host '============================================' -ForegroundColor Green
Write-Host '     All dependencies installed!' -ForegroundColor Green
Write-Host '============================================' -ForegroundColor Green
Write-Host ''
Write-Host 'Run these commands to start the app:' -ForegroundColor Yellow
Write-Host ''
Write-Host '  bun install' -ForegroundColor Cyan
Write-Host '  bun run tauri dev' -ForegroundColor Cyan
Write-Host ''
"@

Start-Process powershell -ArgumentList "-NoExit", "-Command", $startupCommand

# Close current terminal
exit

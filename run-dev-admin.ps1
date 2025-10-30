# Run Tauri dev server with administrator privileges
# This script will restart itself with admin rights if not already running as admin

# Check if running as administrator
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if (-not $isAdmin) {
    Write-Host "Requesting administrator privileges..." -ForegroundColor Yellow
    
    # Get the path to this script
    $scriptPath = $MyInvocation.MyCommand.Path
    
    # Restart this script with admin privileges
    Start-Process powershell.exe -Verb RunAs -ArgumentList "-NoExit", "-ExecutionPolicy", "Bypass", "-File", "`"$scriptPath`""
    
    # Exit the current non-admin instance
    exit
}

# If we get here, we're running as admin
Write-Host "Running with administrator privileges" -ForegroundColor Green
Write-Host "Starting Tauri dev server..." -ForegroundColor Cyan
Write-Host ""

# Change to the script's directory
Set-Location $PSScriptRoot

# Run the Tauri dev server
npm run tauri dev

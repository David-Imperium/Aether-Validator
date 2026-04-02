# Synward MCP Server Installer for Windows
# Usage: irm https://raw.githubusercontent.com/David-Imperium/Synward/main/install.ps1 | iex

param(
    [string]$Version = "latest",
    [string]$InstallDir = "$env:LOCALAPPDATA\Synward",
    [string]$ContractsDir = "",
    [switch]$FactoryCLI = $false,
    [switch]$GeminiCLI = $false,
    [switch]$Help
)

$ErrorActionPreference = "Stop"

# Colors for output
function Write-Info { param($msg) Write-Host "[INFO] $msg" -ForegroundColor Cyan }
function Write-Success { param($msg) Write-Host "[OK] $msg" -ForegroundColor Green }
function Write-Error { param($msg) Write-Host "[ERROR] $msg" -ForegroundColor Red; exit 1 }
function Write-Warning { param($msg) Write-Host "[WARN] $msg" -ForegroundColor Yellow }

# Help
if ($Help) {
    Write-Host @"
Synward MCP Server Installer

Usage: ./install.ps1 [options]

Options:
    -Version       Version to install (default: latest)
    -InstallDir    Installation directory (default: $env:LOCALAPPDATA\Synward)
    -ContractsDir  Custom contracts directory (default: bundled)
    -FactoryCLI    Configure for Factory CLI
    -GeminiCLI     Configure for Gemini CLI
    -Help          Show this help

Examples:
    ./install.ps1 -FactoryCLI
    ./install.ps1 -GeminiCLI -Version v0.1.0
    ./install.ps1 -InstallDir C:\Tools\Synward
"@
    exit 0
}

# Detect architecture
$Arch = if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { "aarch64" } else { "x86_64" }
$Target = "windows-$Arch"

Write-Info "Detected: $Target"

# Get latest version if not specified
if ($Version -eq "latest") {
    Write-Info "Fetching latest version..."
    $LatestRelease = Invoke-RestMethod -Uri "https://api.github.com/repos/David-Imperium/Synward/releases/latest" -ErrorAction SilentlyContinue
    if ($LatestRelease -and $LatestRelease.tag_name) {
        $Version = $LatestRelease.tag_name
    } else {
        $Version = "v0.1.0"
        Write-Warning "Could not fetch latest version, using $Version"
    }
}

Write-Info "Installing Synward MCP $Version"

# Create installation directory
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    Write-Info "Created directory: $InstallDir"
}

# Download binary
$BinaryName = "synward-mcp-server-$Target.exe"
$DownloadUrl = "https://github.com/David-Imperium/Synward/releases/download/$Version/$BinaryName"
$BinaryPath = Join-Path $InstallDir "synward-mcp-server.exe"

Write-Info "Downloading from: $DownloadUrl"

try {
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $BinaryPath -ErrorAction Stop
    Write-Success "Downloaded binary to: $BinaryPath"
} catch {
    Write-Error "Download failed: $_`nYou may need to build from source: cargo build --release"
}

# Extract contracts
$ContractsPath = if ($ContractsDir) { $ContractsDir } else { Join-Path $InstallDir "contracts" }
if (-not (Test-Path $ContractsPath)) {
    New-Item -ItemType Directory -Path $ContractsPath -Force | Out-Null
    
    # Download default contracts
    $ContractsUrl = "https://raw.githubusercontent.com/David-Imperium/Synward/main/contracts/"
    @("rust", "cpp", "lex") | ForEach-Object {
        $LangDir = Join-Path $ContractsPath $_
        New-Item -ItemType Directory -Path $LangDir -Force | Out-Null
    }
    Write-Info "Created contracts directory: $ContractsPath"
}

# Configure Factory CLI
if ($FactoryCLI) {
    Write-Info "Configuring Factory CLI..."
    
    $McpConfigPath = Join-Path $env:USERPROFILE ".factory\mcp.json"
    $McpConfig = @{}
    
    if (Test-Path $McpConfigPath) {
        $McpConfig = Get-Content $McpConfigPath | ConvertFrom-Json -AsHashtable
    }
    
    $McpConfig["mcpServers"] = @{
        "synward" = @{
            "type" = "stdio"
            "command" = $BinaryPath
            "args" = @("--contracts", $ContractsPath)
            "disabled" = $false
        }
    }
    
    $McpConfig | ConvertTo-Json -Depth 10 | Set-Content $McpConfigPath
    Write-Success "Configured Factory CLI: $McpConfigPath"
}

# Configure Gemini CLI
if ($GeminiCLI) {
    Write-Info "Configuring Gemini CLI..."
    
    $GeminiConfigPath = Join-Path $env:USERPROFILE ".gemini\settings.json"
    $GeminiConfig = @{}
    
    if (Test-Path $GeminiConfigPath) {
        $GeminiConfig = Get-Content $GeminiConfigPath | ConvertFrom-Json -AsHashtable
    }
    
    $GeminiConfig["mcpServers"] = @{
        "synward" = @{
            "command" = $BinaryPath
            "args" = @("--contracts", $ContractsPath)
        }
    }
    
    $GeminiConfig | ConvertTo-Json -Depth 10 | Set-Content $GeminiConfigPath
    Write-Success "Configured Gemini CLI: $GeminiConfigPath"
}

# Verify installation
Write-Info "Verifying installation..."
$TestResult = & $BinaryPath --version 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Success "Synward MCP Server installed successfully!"
    Write-Host ""
    Write-Host "Binary: $BinaryPath"
    Write-Host "Contracts: $ContractsPath"
    Write-Host ""
    
    if (-not $FactoryCLI -and -not $GeminiCLI) {
        Write-Host "To configure your AI client, add to your MCP config:"
        Write-Host @"
{
  "mcpServers": {
    "synward": {
      "type": "stdio",
      "command": "$BinaryPath",
      "args": ["--contracts", "$ContractsPath"],
      "disabled": false
    }
  }
}
"@
    }
} else {
    Write-Error "Installation verification failed"
}

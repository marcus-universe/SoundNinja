<#
.SYNOPSIS
Downloads the Steinberg ASIO SDK into vendor/asiosdk and points bindgen at libclang.

.DESCRIPTION
cpal/asio-sys needs CPAL_ASIO_DIR at compile time. This repo points that
variable at vendor/asiosdk via .cargo/config.toml. The SDK itself must not
be committed (Steinberg license). Run once on Windows before tauri:serve.

asio-sys bindgen also needs libclang.dll. This script finds LLVM (or installs
it via scoop/winget), junctions vendor/libclang, and sets the User
LIBCLANG_PATH. Cargo must not set LIBCLANG_PATH globally — that breaks
Linux/macOS bindgen when vendor/libclang is missing.

.EXAMPLE
bun run setup:asio
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/setup-asio-sdk.ps1
#>
$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$vendor = Join-Path $repoRoot "vendor"
$dest = Join-Path $vendor "asiosdk"
$marker = Join-Path $dest "common"
$libclangLink = Join-Path $vendor "libclang"

function Test-AsioRoot([string]$path) {
  return (Test-Path (Join-Path $path "common")) -and (Test-Path (Join-Path $path "host"))
}

function Find-LibClangDir {
  $candidates = [System.Collections.Generic.List[string]]::new()
  if ($env:LIBCLANG_PATH) { [void]$candidates.Add($env:LIBCLANG_PATH) }
  $candidates.Add((Join-Path $env:USERPROFILE "scoop\apps\llvm\current\bin"))
  $candidates.Add("C:\Program Files\LLVM\bin")
  $candidates.Add("C:\Program Files (x86)\LLVM\bin")
  $candidates.Add("C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\Llvm\x64\bin")
  $candidates.Add("C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Tools\Llvm\x64\bin")
  foreach ($dir in $candidates) {
    if ($dir -and (Test-Path (Join-Path $dir "libclang.dll"))) {
      return (Resolve-Path $dir).Path
    }
  }
  return $null
}

function Install-LibClang {
  $dir = Find-LibClangDir
  if ($dir) { return $dir }

  Write-Host "libclang.dll not found. Installing LLVM for asio-sys bindgen..."
  if (Get-Command scoop -ErrorAction SilentlyContinue) {
    scoop install llvm
  } elseif (Get-Command winget -ErrorAction SilentlyContinue) {
    winget install --id LLVM.LLVM -e --accept-package-agreements --accept-source-agreements
  } else {
    throw "Install LLVM (https://github.com/llvm/llvm-project/releases) so libclang.dll is on disk, then re-run bun run setup:asio."
  }

  $dir = Find-LibClangDir
  if (-not $dir) {
    throw "LLVM installed but libclang.dll still not found. Set LIBCLANG_PATH to the folder that contains it."
  }
  return $dir
}

function Link-LibClang([string]$fromDir) {
  New-Item -ItemType Directory -Force -Path $vendor | Out-Null
  if (Test-Path $libclangLink) {
    cmd /c rmdir "$libclangLink" > $null 2>&1
    if (Test-Path $libclangLink) {
      Remove-Item -Recurse -Force $libclangLink
    }
  }
  cmd /c mklink /J "$libclangLink" "$fromDir" | Out-Null
  if (-not (Test-Path (Join-Path $libclangLink "libclang.dll"))) {
    throw "Failed to junction $libclangLink -> $fromDir"
  }
  Write-Host "LIBCLANG_PATH junction: $libclangLink -> $fromDir"
}

# --- Steinberg SDK ---
if (Test-AsioRoot $dest) {
  Write-Host "ASIO SDK already at $dest"
} else {
  New-Item -ItemType Directory -Force -Path $vendor | Out-Null
  $zip = Join-Path $env:TEMP "soundninja-asiosdk.zip"
  $extract = Join-Path $env:TEMP "soundninja-asiosdk"
  if (Test-Path $extract) {
    Remove-Item -Recurse -Force $extract
  }

  $urls = @(
    "https://download.steinberg.net/sdk_downloads/ASIO-SDK_2.3.4_2025-10-15.zip",
    "https://www.steinberg.net/asiosdk"
  )

  $ok = $false
  foreach ($url in $urls) {
    try {
      Write-Host "Downloading ASIO SDK from $url"
      Invoke-WebRequest -Uri $url -OutFile $zip -UseBasicParsing
      $ok = $true
      break
    } catch {
      Write-Host "Download failed: $_"
    }
  }

  if (-not $ok) {
    throw "Could not download Steinberg ASIO SDK. Download it from https://www.steinberg.net/asiosdk and extract so vendor/asiosdk contains common/ and host/."
  }

  Expand-Archive -Path $zip -DestinationPath $extract -Force

  $sdkRoot = $null
  if (Test-AsioRoot $extract) {
    $sdkRoot = $extract
  } else {
    $sdkRoot = Get-ChildItem $extract -Directory -ErrorAction SilentlyContinue |
      Where-Object { Test-AsioRoot $_.FullName } |
      Select-Object -First 1 -ExpandProperty FullName
  }

  if (-not $sdkRoot) {
    throw "ZIP extracted but no ASIO SDK root (common/ + host/) found in $extract"
  }

  if (Test-Path $dest) {
    Remove-Item -Recurse -Force $dest
  }
  Copy-Item -Recurse -Force $sdkRoot $dest
  Write-Host "ASIO SDK installed at $dest"
}

# --- libclang for bindgen ---
$clangDir = Install-LibClang
Link-LibClang $clangDir
$env:LIBCLANG_PATH = $libclangLink
[Environment]::SetEnvironmentVariable("LIBCLANG_PATH", $libclangLink, "User")
Write-Host "User LIBCLANG_PATH=$libclangLink (new terminals pick this up)"
Write-Host "Next: bun run tauri:serve"

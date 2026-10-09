<#
.SYNOPSIS
Generate GPG / AUR SSH keys into .env and optionally push them as GitHub secrets.

.DESCRIPTION
Never commits .env. Workflows do not read .env; CI uses GitHub Actions secrets
with the same names. Existing non-empty .env values are not overwritten.

.EXAMPLE
./scripts/setup-secrets.ps1 -GenerateGpg -GenerateAurSsh
./scripts/setup-secrets.ps1 -Push
#>
[CmdletBinding()]
param(
    [switch]$GenerateGpg,
    [switch]$GenerateAurSsh,
    [switch]$Push,
    [switch]$InstallSshConfig,
    [string]$Repo = "marcus-universe/SoundNinja"
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$envExample = Join-Path $repoRoot ".env.example"
$envFile = Join-Path $repoRoot ".env"
$GpgUid = "Marcus Universe <contact@marcus-universe.de>"
$PushSkip = @(
    "GPG_PUBLIC_KEY",
    "AUR_SSH_PUBLIC_KEY",
    "GPG_REVOCATION_CERT"
)

function Get-Gpg {
    $cmd = Get-Command gpg -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Source }
    $candidates = @(
        "C:\Program Files\Git\usr\bin\gpg.exe",
        "C:\Program Files (x86)\GnuPG\bin\gpg.exe",
        "C:\Program Files\GnuPG\bin\gpg.exe"
    )
    foreach ($p in $candidates) {
        if (Test-Path $p) { return $p }
    }
    throw "gpg not found. Install Gpg4win or Git for Windows."
}

function Get-DotEnvRegex {
    # KEY=value or KEY="value with \n, \" and \\ escapes" on a single line.
    return [regex]::new('^(?:export\s+)?([A-Z0-9_]+)=("(?:[^"\\\r\n]|\\.)*"|[^\r\n]*)', 'Multiline')
}

function Read-DotEnv([string]$path) {
    $map = [ordered]@{}
    if (-not (Test-Path $path)) { return $map }
    $raw = [System.IO.File]::ReadAllText($path)
    foreach ($m in (Get-DotEnvRegex).Matches($raw)) {
        $key = $m.Groups[1].Value
        $val = $m.Groups[2].Value
        if ($val.StartsWith('"') -and $val.EndsWith('"') -and $val.Length -ge 2) {
            $val = [regex]::Replace($val.Substring(1, $val.Length - 2), '\\(.)', {
                param($e)
                if ($e.Groups[1].Value -eq 'n') { return "`n" }
                return $e.Groups[1].Value
            })
        }
        $map[$key] = $val
    }
    return $map
}

function Format-DotEnvLine([string]$key, [string]$val) {
    if ($val -match "[\r\n]" -or $val -match '\s' -or $val -match '"') {
        $escaped = $val.Replace('\', '\\').Replace('"', '\"').Replace("`r`n", "`n").Replace("`n", '\n')
        return "$key=`"$escaped`""
    }
    return "$key=$val"
}

# Rewrites only KEY=value entries this script manages; every other line in an
# existing .env (comments, hand-written entries) is kept verbatim.
function Write-DotEnv($map, [string]$path) {
    $raw = ""
    if (Test-Path $path) { $raw = [System.IO.File]::ReadAllText($path) }
    if ([string]::IsNullOrWhiteSpace($raw)) {
        $raw = "# Generated/updated by scripts/setup-secrets.ps1. DO NOT COMMIT.`n" +
            "# Backup encrypted only (KeePassXC / Bitwarden / encrypted archive).`n`n"
    }

    $written = [System.Collections.Generic.HashSet[string]]::new()
    $out = (Get-DotEnvRegex).Replace($raw, {
        param($m)
        $key = $m.Groups[1].Value
        if (-not $map.Contains($key) -or -not $written.Add($key)) { return $m.Value }
        return Format-DotEnvLine $key ([string]$map[$key])
    })

    $sb = [System.Text.StringBuilder]::new($out)
    if ($out.Length -gt 0 -and -not $out.EndsWith("`n")) { [void]$sb.Append("`n") }
    foreach ($key in $map.Keys) {
        if ($written.Contains($key)) { continue }
        [void]$sb.Append((Format-DotEnvLine $key ([string]$map[$key])) + "`n")
    }

    $utf8 = New-Object System.Text.UTF8Encoding $false
    [System.IO.File]::WriteAllText($path, $sb.ToString(), $utf8)
    Write-Host "Wrote $path"
}

function Set-IfEmpty($map, [string]$key, [string]$value) {
    if (-not $map.Contains($key) -or [string]::IsNullOrWhiteSpace([string]$map[$key])) {
        $map[$key] = $value
        return $true
    }
    Write-Host "Skip $key (already set)"
    return $false
}

function New-RandomPassphrase {
    $bytes = New-Object byte[] 24
    [System.Security.Cryptography.RandomNumberGenerator]::Create().GetBytes($bytes)
    return [Convert]::ToBase64String($bytes)
}

if (-not (Test-Path $envFile)) {
    if (-not (Test-Path $envExample)) { throw "Missing $envExample" }
    Copy-Item $envExample $envFile
    Write-Host "Created $envFile from .env.example"
}

$map = Read-DotEnv $envFile

if ($GenerateGpg) {
    $gpg = Get-Gpg
    if (-not [string]::IsNullOrWhiteSpace([string]$map["GPG_PRIVATE_KEY"])) {
        Write-Host "Skip GPG generate (GPG_PRIVATE_KEY already set)"
    } else {
        $pass = New-RandomPassphrase
        # Isolated keyring: a fresh default homedir gets "use-keyboxd", which
        # Git for Windows' gpg cannot start. Secrets end up in .env anyway.
        $gpgHome = Join-Path ([System.IO.Path]::GetTempPath()) ("gpg-" + [guid]::NewGuid().ToString("n"))
        New-Item -ItemType Directory -Path $gpgHome | Out-Null
        [System.IO.File]::WriteAllText((Join-Path $gpgHome "common.conf"), "")
        $gpgHomeArg = $gpgHome -replace '\\', '/'
        if ($gpg -like "*\Git\usr\bin\*" -and $gpgHomeArg -match '^([A-Za-z]):(.*)$') {
            # MSYS gpg only spawns gpg-agent with a POSIX homedir.
            $gpgHomeArg = "/" + $Matches[1].ToLower() + $Matches[2]
        }
        $gpgArgs = @("--homedir", $gpgHomeArg)
        try {
            Write-Host "Generating GPG key for $GpgUid ..."
            & $gpg @gpgArgs --batch --pinentry-mode loopback --passphrase $pass --quick-gen-key $GpgUid ed25519 sign 3y
            if ($LASTEXITCODE -ne 0) { throw "gpg --quick-gen-key failed" }

            $fpr = (& $gpg @gpgArgs --batch --with-colons --list-secret-keys --fingerprint $GpgUid |
                Where-Object { $_ -like "fpr:*" } |
                Select-Object -First 1)
            if (-not $fpr) { throw "Could not read GPG fingerprint" }
            $fingerprint = ($fpr -split ":")[9]
            if ($fingerprint.Length -ne 40) { throw "Unexpected fingerprint: $fingerprint" }

            $privateKey = & $gpg @gpgArgs --batch --pinentry-mode loopback --passphrase $pass --armor --export-secret-keys $fingerprint
            if ($LASTEXITCODE -ne 0 -or -not $privateKey) { throw "gpg --export-secret-keys failed" }
            $publicKey = & $gpg @gpgArgs --batch --armor --export $fingerprint
            if ($LASTEXITCODE -ne 0 -or -not $publicKey) { throw "gpg --export failed" }

            $revokeInput = "y`n0`nSoundNinja release-signing revocation certificate.`ny`n"
            $prevPref = $ErrorActionPreference
            $ErrorActionPreference = "Continue"
            $revocation = $revokeInput | & $gpg @gpgArgs --batch --yes --pinentry-mode loopback --passphrase $pass --command-fd 0 --status-fd 2 --gen-revoke $fingerprint 2>$null
            $ErrorActionPreference = $prevPref
            $autoRev = Join-Path $gpgHome "openpgp-revocs.d\$fingerprint.rev"
            if (-not $revocation -and (Test-Path $autoRev)) {
                $revocation = Get-Content -Raw $autoRev
            }
            if (-not $revocation) {
                Write-Warning "Could not generate revocation certificate automatically. Import GPG_PRIVATE_KEY and run: gpg --gen-revoke $fingerprint"
                $revocation = ""
            }
        } finally {
            $gpgconf = Join-Path (Split-Path $gpg) "gpgconf.exe"
            if (Test-Path $gpgconf) { try { & $gpgconf @gpgArgs --kill all 2>$null | Out-Null } catch { } }
            Remove-Item -Recurse -Force $gpgHome -ErrorAction SilentlyContinue
        }

        $map["GPG_PRIVATE_KEY"] = (($privateKey | Out-String).Trim())
        $map["GPG_PASSPHRASE"] = $pass
        $map["GPG_FINGERPRINT"] = $fingerprint
        $map["GPG_PUBLIC_KEY"] = (($publicKey | Out-String).Trim())
        $map["GPG_REVOCATION_CERT"] = (($revocation | Out-String).Trim())
        Write-Host "GPG fingerprint $fingerprint"
    }
}

if ($GenerateAurSsh) {
    if (-not [string]::IsNullOrWhiteSpace([string]$map["AUR_SSH_PRIVATE_KEY"])) {
        Write-Host "Skip AUR SSH generate (AUR_SSH_PRIVATE_KEY already set)"
    } else {
        $ssh = Get-Command ssh-keygen -ErrorAction SilentlyContinue
        if (-not $ssh) { throw "ssh-keygen not found (OpenSSH)." }
        $dir = Join-Path ([System.IO.Path]::GetTempPath()) ("aur-ssh-" + [guid]::NewGuid().ToString("n"))
        New-Item -ItemType Directory -Path $dir | Out-Null
        try {
            $keyFile = Join-Path $dir "aur"
            & $ssh.Source -t ed25519 -q -N "" -C "aur-soundninja" -f $keyFile
            if ($LASTEXITCODE -ne 0) { throw "ssh-keygen failed" }
            $map["AUR_SSH_PRIVATE_KEY"] = (Get-Content -Raw $keyFile).Trim()
            $map["AUR_SSH_PUBLIC_KEY"] = (Get-Content -Raw "$keyFile.pub").Trim()
            Write-Host "AUR SSH public key:"
            Write-Host $map["AUR_SSH_PUBLIC_KEY"]
            if ($InstallSshConfig) {
                $sshDir = Join-Path $env:USERPROFILE ".ssh"
                New-Item -ItemType Directory -Force -Path $sshDir | Out-Null
                $aurKey = Join-Path $sshDir "aur"
                Copy-Item $keyFile $aurKey -Force
                Copy-Item "$keyFile.pub" "$aurKey.pub" -Force
                $cfg = Join-Path $sshDir "config"
                $block = @"
Host aur.archlinux.org
  IdentityFile ~/.ssh/aur
  User aur
"@
                if (-not (Test-Path $cfg) -or -not (Select-String -Path $cfg -Pattern "aur.archlinux.org" -Quiet)) {
                    Add-Content -Path $cfg -Value "`n$block" -Encoding utf8
                    Write-Host "Appended Host aur.archlinux.org to $cfg"
                }
            }
        } finally {
            Remove-Item -Recurse -Force $dir -ErrorAction SilentlyContinue
        }
    }
}

Set-IfEmpty $map "AUR_EMAIL" "contact@marcus-universe.de" | Out-Null
Write-DotEnv $map $envFile

if ($Push) {
    if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
        throw "gh CLI not found. Install GitHub CLI and authenticate."
    }
    $toPush = @(
        "GPG_PRIVATE_KEY",
        "GPG_PASSPHRASE",
        "GPG_FINGERPRINT",
        "TAURI_SIGNING_PRIVATE_KEY",
        "TAURI_SIGNING_PRIVATE_KEY_PASSWORD",
        "AUR_USERNAME",
        "AUR_EMAIL",
        "AUR_SSH_PRIVATE_KEY"
    )
    foreach ($key in $toPush) {
        if ($PushSkip -contains $key) { continue }
        $val = [string]$map[$key]
        if ([string]::IsNullOrWhiteSpace($val)) {
            Write-Host "Skip secret $key (empty)"
            continue
        }
        $val | & gh secret set $key --repo $Repo
        if ($LASTEXITCODE -ne 0) { throw "gh secret set $key failed" }
        Write-Host "Set GitHub secret $key"
    }
    Write-Host "Done. Public keys stay local: GPG_PUBLIC_KEY, AUR_SSH_PUBLIC_KEY, GPG_REVOCATION_CERT"
}

if (-not $GenerateGpg -and -not $GenerateAurSsh -and -not $Push) {
    Write-Host "No action. Use -GenerateGpg, -GenerateAurSsh, and/or -Push."
}

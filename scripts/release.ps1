param(
    [Parameter(Mandatory)][string]$Version
)

$ErrorActionPreference = "Stop"

if ($Version -notmatch '^\d+\.\d+\.\d+$') {
    Write-Error "Usage: .\scripts\release.ps1 0.1.2"
}

$tag = "v$Version"

Write-Host "Releasing $tag..." -ForegroundColor Cyan

# Bump versions
(Get-Content src-tauri\Cargo.toml)       -replace '^version = "\d+\.\d+\.\d+"', "version = `"$Version`"" | Set-Content src-tauri\Cargo.toml
(Get-Content src-tauri\tauri.conf.json)  -replace '"version": "\d+\.\d+\.\d+"',  "`"version`": `"$Version`""                                    | Set-Content src-tauri\tauri.conf.json
(Get-Content crates\devpane\Cargo.toml)  -replace '^version = "\d+\.\d+\.\d+"', "version = `"$Version`"" | Set-Content crates\devpane\Cargo.toml
(Get-Content crates\cli\Cargo.toml)      -replace '^version = "\d+\.\d+\.\d+"', "version = `"$Version`"" | Set-Content crates\cli\Cargo.toml

# Update Cargo.lock
cargo update --manifest-path src-tauri/Cargo.toml --workspace 2>&1 | Out-Null

# Commit + tag + push
git add src-tauri/Cargo.toml src-tauri/tauri.conf.json crates/devpane/Cargo.toml crates/cli/Cargo.toml Cargo.lock
git commit -m "Release $tag"
git tag $tag
git push origin master --tags

Write-Host "Done — GitHub Actions is building $tag" -ForegroundColor Green

# Windows Build Script for Forma Fractalis
# Creates a distribution package with executable and license files

$ErrorActionPreference = "Stop"

Write-Host "Building Forma Fractalis for Windows (release mode)..." -ForegroundColor Cyan

# Remap all user-specific build paths so they don't appear in panic messages.
# Covers: project source, cargo registry, and rustup stdlib paths.
$projectRoot = (Get-Location).Path
$cargoHome = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { "$env:USERPROFILE\.cargo" }
$rustupHome = if ($env:RUSTUP_HOME) { $env:RUSTUP_HOME } else { "$env:USERPROFILE\.rustup" }
$env:RUSTFLAGS = "--remap-path-prefix=${projectRoot}=. --remap-path-prefix=${cargoHome}=<cargo> --remap-path-prefix=${rustupHome}=<rustup>"

# Build the release executable
cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host "Build successful!" -ForegroundColor Green

# Create builds directory if it doesn't exist
$buildsDir = "builds"
if (-not (Test-Path $buildsDir)) {
    New-Item -ItemType Directory -Path $buildsDir | Out-Null
    Write-Host "Created builds directory" -ForegroundColor Yellow
}

# Get version from Cargo.toml
$cargoToml = Get-Content "Cargo.toml" -Raw
if ($cargoToml -match 'version\s*=\s*"([^"]+)"') {
    $version = $matches[1]
} else {
    $version = "unknown"
}

# Create distribution directory name
$distName = "forma-fractalis_v${version}_windows"
$distPath = Join-Path $buildsDir $distName

# Remove old distribution directory if it exists
if (Test-Path $distPath) {
    Remove-Item -Path $distPath -Recurse -Force
}

# Create distribution directory
New-Item -ItemType Directory -Path $distPath | Out-Null

Write-Host "Copying files to distribution directory..." -ForegroundColor Cyan

# Copy executable
Copy-Item "target\release\forma-fractalis.exe" -Destination $distPath

# Copy license files
Copy-Item "LICENSE-APACHE" -Destination (Join-Path $distPath "LICENSE-APACHE.txt")
Copy-Item "LICENSE-MIT" -Destination (Join-Path $distPath "LICENSE-MIT.txt")
Copy-Item "THIRD_PARTY_LICENSES.md" -Destination $distPath

# Copy documentation files
Copy-Item "README.md" -Destination $distPath
Copy-Item "COLORMAP_SAVELOAD.md" -Destination $distPath

# Create zip file
$zipPath = Join-Path $buildsDir "$distName.zip"
if (Test-Path $zipPath) {
    Remove-Item $zipPath -Force
}

Write-Host "Creating zip archive..." -ForegroundColor Cyan
# Use 7z with metadata-cleaning flags - cd into builds to avoid nested paths
Push-Location $buildsDir
7z a -tzip -mtc=off -mta=off "$distName.zip" "$distName\*"
Pop-Location

# Clean up distribution directory
Remove-Item -Path $distPath -Recurse -Force

Write-Host ""
Write-Host "Distribution created successfully!" -ForegroundColor Green
Write-Host "Location: $zipPath" -ForegroundColor Yellow
Write-Host ""
Write-Host "Contents:" -ForegroundColor Cyan
Write-Host "  - forma-fractalis.exe"
Write-Host "  - LICENSE-APACHE.txt"
Write-Host "  - LICENSE-MIT.txt"
Write-Host "  - THIRD_PARTY_LICENSES.md"
Write-Host "  - README.md"
Write-Host "  - COLORMAP_SAVELOAD.md"

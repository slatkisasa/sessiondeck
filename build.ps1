param([string]$Configuration = "Release")
$ErrorActionPreference = "Stop"
Push-Location $PSScriptRoot
try {
    $target = "x86_64-pc-windows-msvc"
    $profile = if ($Configuration -eq "Release") { "release" } else { "dev" }
    & cargo build --locked --profile $profile --target $target
    if ($LASTEXITCODE -ne 0) { throw "Cargo build failed." }
    $folder = if ($profile -eq "dev") { "debug" } else { $profile }
    New-Item -ItemType Directory -Force dist | Out-Null
    Copy-Item "target\$target\$folder\SessionDeck.exe" "dist\SessionDeck.exe"
    Write-Host "Portable executable: dist\SessionDeck.exe"
} finally { Pop-Location }

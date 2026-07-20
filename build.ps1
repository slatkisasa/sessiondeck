param(
    [string]$Configuration = "Release"
)

$ErrorActionPreference = "Stop"
$projectRoot = $PSScriptRoot
$localDotnet = Join-Path $projectRoot ".dotnet-sdk\dotnet.exe"
$dotnet = if (Test-Path -LiteralPath $localDotnet) { $localDotnet } else { "dotnet" }
$env:DOTNET_CLI_HOME = Join-Path $projectRoot ".dotnet-cli-home"
$env:NUGET_PACKAGES = Join-Path $projectRoot ".nuget-packages"
$env:DOTNET_SKIP_FIRST_TIME_EXPERIENCE = "1"
$env:DOTNET_CLI_TELEMETRY_OPTOUT = "1"

& $dotnet publish (Join-Path $projectRoot "BNetSwitcher.csproj") `
    --configuration $Configuration `
    --runtime win-x64 `
    --self-contained true `
    --output (Join-Path $projectRoot "dist")

if ($LASTEXITCODE -ne 0) {
    throw "The build failed with exit code $LASTEXITCODE."
}

Write-Host "Portable executable: $(Join-Path $projectRoot 'dist\BNetSwitcher.exe')"

param(
    [string]$Version = "1.12.93",
    [string]$Target = "x86_64-pc-windows-gnu"
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

cargo build --release --target $Target

$Iscc = Get-Command ISCC.exe -ErrorAction SilentlyContinue
if (-not $Iscc) {
    $Iscc = Get-Command iscc -ErrorAction SilentlyContinue
}
if (-not $Iscc) {
    throw "Inno Setup compiler (ISCC.exe) was not found in PATH"
}

New-Item -ItemType Directory -Force -Path (Join-Path $Root "dist") | Out-Null
& $Iscc.Source "/DMyAppVersion=$Version" "installer\entropy.iss"

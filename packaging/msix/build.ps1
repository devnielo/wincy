# Packs wincy.exe into an UNSIGNED Microsoft Store package (Partner Center signs it). See docs/STORE.md.
#   ./packaging/msix/build.ps1 -Version 0.3.0
# Output: <OutDir>\Wincy_X.Y.Z.0_x64.msix
param(
  [Parameter(Mandatory)][string]$Version, # X.Y.Z (a leading "v" is ignored)
  [string]$Exe = "$PSScriptRoot\..\..\src-tauri\target\release\wincy.exe",
  [string]$OutDir = "$PSScriptRoot\..\..\src-tauri\target\msix"
)
$ErrorActionPreference = 'Stop'

$v = $Version.TrimStart('v')
if ($v -notmatch '^\d+\.\d+\.\d+$') { throw "Version must be X.Y.Z, got '$Version'" }
$full = "$v.0"

# makeappx/makepri: the newest Windows SDK, else the Microsoft.Windows.SDK.BuildTools NuGet package.
function Find-SdkBin {
  $kits = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits\10\bin'
  $sdk = Get-ChildItem $kits -Directory -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -match '^10\.[\d.]+$' -and (Test-Path "$($_.FullName)\x64\makeappx.exe") -and (Test-Path "$($_.FullName)\x64\makepri.exe") } |
    Sort-Object { [version]$_.Name } -Descending | Select-Object -First 1
  if ($sdk) { return "$($sdk.FullName)\x64" }

  $tools = Join-Path ([IO.Path]::GetTempPath()) 'wincy-sdk-buildtools'
  $found = { Get-ChildItem $tools -Recurse -Filter makeappx.exe -ErrorAction SilentlyContinue | Where-Object { $_.Directory.Name -eq 'x64' } | Select-Object -First 1 }
  $bin = & $found
  if (-not $bin) {
    Write-Host 'Windows SDK not found; downloading Microsoft.Windows.SDK.BuildTools from NuGet'
    Remove-Item $tools -Recurse -Force -ErrorAction SilentlyContinue
    New-Item -ItemType Directory $tools | Out-Null
    $zip = Join-Path $tools 'buildtools.zip'
    Invoke-WebRequest 'https://www.nuget.org/api/v2/package/Microsoft.Windows.SDK.BuildTools' -OutFile $zip -UseBasicParsing
    Expand-Archive $zip (Join-Path $tools 'pkg')
    $bin = & $found
    if (-not $bin) { throw 'makeappx.exe (x64) not found in Microsoft.Windows.SDK.BuildTools' }
  }
  return $bin.DirectoryName
}

function Invoke-Tool([string]$Path, [string[]]$ToolArgs) {
  & $Path @ToolArgs
  if ($LASTEXITCODE) { throw "$(Split-Path $Path -Leaf) failed with exit code $LASTEXITCODE" }
}

if (-not (Test-Path $Exe -PathType Leaf)) { throw "wincy.exe not found: $Exe" }
$bin = Find-SdkBin
Write-Host "Using SDK tools in $bin"

New-Item -ItemType Directory $OutDir -Force | Out-Null
$OutDir = (Resolve-Path $OutDir).Path
$stage = Join-Path $OutDir 'stage'
$work = Join-Path $OutDir 'work'
Remove-Item $stage, $work -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory "$stage\Assets", $work | Out-Null

Copy-Item $Exe "$stage\wincy.exe"
foreach ($logo in 'Square44x44Logo.png', 'Square150x150Logo.png', 'StoreLogo.png') {
  Copy-Item "$PSScriptRoot\..\..\src-tauri\icons\$logo" "$stage\Assets\$logo"
}
$manifest = (Get-Content "$PSScriptRoot\AppxManifest.xml" -Raw).Replace('{{VERSION}}', $full)
[IO.File]::WriteAllText("$stage\AppxManifest.xml", $manifest) # UTF-8 without BOM

# resources.pri: default language first (en-US), then es-ES.
Invoke-Tool "$bin\makepri.exe" @('createconfig', '/cf', "$work\priconfig.xml", '/dq', 'en-US_es-ES', '/pv', '10.0.0', '/o')
Invoke-Tool "$bin\makepri.exe" @('new', '/pr', $stage, '/cf', "$work\priconfig.xml", '/mn', "$stage\AppxManifest.xml", '/of', "$stage\resources.pri", '/o')

$msix = Join-Path $OutDir "Wincy_${full}_x64.msix"
Invoke-Tool "$bin\makeappx.exe" @('pack', '/d', $stage, '/p', $msix, '/o')
Write-Host "Created $msix"

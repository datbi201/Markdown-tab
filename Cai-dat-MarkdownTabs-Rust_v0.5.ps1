# =====================================================================
#  Markdown Tabs (Rust native) v0.5 - Trinh cai dat per-user cho Windows
#  - Cai vao: %LOCALAPPDATA%\Programs\Markdown Tabs (ghi de ban Python
#    neu da cai v1.2 o cung thu muc - shortcut/lien ket cu van dung duoc
#    vi cung ten file MarkdownTabs.exe)
#  - Chi la 1 file .exe native: khong can Python, khong can cai them gi
#  - Dang ky .md/.markdown/.mdown + dat mac dinh (hoi Y/N nhu ban truoc)
#  Tham so: -NoDesktopShortcut  -SetDefault  -Silent
# =====================================================================
param(
    [switch]$NoDesktopShortcut,
    [switch]$SetDefault,
    [switch]$Silent
)
$ErrorActionPreference = "Stop"
$AppName    = "Markdown Tabs"
$Version    = "0.5.0-rust"
$ProgId     = "MarkdownTabs.Document"
$ExeName    = "MarkdownTabs.exe"
$IconName   = "MarkdownTabs_v1.2.ico"
$Exts       = @(".md", ".markdown", ".mdown")
$PkgDir     = $PSScriptRoot
$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\Markdown Tabs"

function Say($msg) { Write-Host $msg }
function Set-RegString($path, $name, $value) {
    if (-not (Test-Path $path)) { New-Item -Path $path -Force | Out-Null }
    if ($name -eq "(Default)") { Set-Item -Path $path -Value $value }
    else { New-ItemProperty -Path $path -Name $name -Value $value -PropertyType String -Force | Out-Null }
}
function Set-RegDword($path, $name, $value) {
    if (-not (Test-Path $path)) { New-Item -Path $path -Force | Out-Null }
    New-ItemProperty -Path $path -Name $name -Value $value -PropertyType DWord -Force | Out-Null
}
function Add-OpenWithProgId($ext) {
    $key = [Microsoft.Win32.Registry]::CurrentUser.CreateSubKey("Software\Classes\$ext\OpenWithProgids")
    try { $key.SetValue($ProgId, [byte[]]@(), [Microsoft.Win32.RegistryValueKind]::None) }
    finally { $key.Close() }
}
function Refresh-Associations() {
    try {
        $sig = '[DllImport("shell32.dll")] public static extern void SHChangeNotify(int wEventId, int uFlags, IntPtr dwItem1, IntPtr dwItem2);'
        $t = Add-Type -MemberDefinition $sig -Name ShellNotifyR -Namespace Win32 -PassThru
        $t::SHChangeNotify(0x08000000, 0, [IntPtr]::Zero, [IntPtr]::Zero)
    } catch { }
}

Say ""
Say "=== Cai dat $AppName ban Rust native v$Version ==="
Say "Thu muc cai: $InstallDir"
Say ""

if (-not (Test-Path (Join-Path $PkgDir $ExeName))) {
    Say "LOI: Thieu file $ExeName trong goi cai dat. Hay giai nen TOAN BO file zip roi chay lai."
    exit 1
}

# Dung instance dang chay de khong bi khoa file khi copy de
$running = Get-Process -Name "MarkdownTabs" -ErrorAction SilentlyContinue
if ($running) {
    Say "Dang dong Markdown Tabs dang chay de cap nhat..."
    $running | Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 600
}

New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
Copy-Item (Join-Path $PkgDir $ExeName) (Join-Path $InstallDir $ExeName) -Force
if (Test-Path (Join-Path $PkgDir $IconName)) { Copy-Item (Join-Path $PkgDir $IconName) (Join-Path $InstallDir $IconName) -Force }
foreach ($f in @("Go-cai-dat-MarkdownTabs-Rust_v0.5.ps1", "Go-cai-dat-MarkdownTabs-Rust_v0.5.bat")) {
    $src = Join-Path $PkgDir $f
    if (Test-Path $src) { Copy-Item $src (Join-Path $InstallDir $f) -Force }
}
$ExePath = Join-Path $InstallDir $ExeName
Say "Da copy MarkdownTabs.exe (ban Rust native)."

# Shortcut
function New-AppShortcut($linkPath) {
    $dir = Split-Path $linkPath -Parent
    if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Path $dir -Force | Out-Null }
    $ws = New-Object -ComObject WScript.Shell
    $sc = $ws.CreateShortcut($linkPath)
    $sc.TargetPath = $ExePath
    $sc.Arguments = ""
    $sc.WorkingDirectory = $InstallDir
    $sc.IconLocation = "$ExePath,0"
    $sc.Description = "Markdown Tabs (Rust native) - xem & soan thao Markdown da tab"
    $sc.Save()
}
$startMenuDir = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\Markdown Tabs"
New-AppShortcut (Join-Path $startMenuDir "Markdown Tabs.lnk")
if (-not $NoDesktopShortcut) {
    New-AppShortcut (Join-Path ([Environment]::GetFolderPath("Desktop")) "Markdown Tabs.lnk")
}
Say "Da tao shortcut Start Menu" + $(if (-not $NoDesktopShortcut) { " va Desktop." } else { "." })

# Dang ky lien ket file
$OpenCmd = "`"$ExePath`" `"%1`""
Set-RegString "HKCU:\Software\Classes\$ProgId" "(Default)" "Markdown Document"
Set-RegString "HKCU:\Software\Classes\$ProgId\DefaultIcon" "(Default)" "$ExePath,0"
Set-RegString "HKCU:\Software\Classes\$ProgId\shell" "(Default)" "open"
Set-RegString "HKCU:\Software\Classes\$ProgId\shell\open\command" "(Default)" $OpenCmd
foreach ($ext in $Exts) { Add-OpenWithProgId $ext }
$capPath = "HKCU:\Software\LisaTools\MarkdownTabs\Capabilities"
Set-RegString $capPath "ApplicationName" $AppName
Set-RegString $capPath "ApplicationDescription" "Trinh xem & soan thao Markdown da tab (Rust native)"
foreach ($ext in $Exts) { Set-RegString "$capPath\FileAssociations" $ext $ProgId }
Set-RegString "HKCU:\Software\RegisteredApplications" $AppName "Software\LisaTools\MarkdownTabs\Capabilities"
$appKey = "HKCU:\Software\Classes\Applications\MarkdownTabs.exe"
Set-RegString "$appKey\shell\open\command" "(Default)" $OpenCmd
Set-RegString "$appKey" "FriendlyAppName" $AppName
foreach ($ext in $Exts) { Set-RegString "$appKey\SupportedTypes" $ext "" }
Say "Da dang ky Markdown Tabs cho .md, .markdown, .mdown."

# Muc go cai dat
$uninst = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\MarkdownTabs"
Set-RegString $uninst "DisplayName" "$AppName (Rust native)"
Set-RegString $uninst "DisplayVersion" $Version
Set-RegString $uninst "Publisher" "LisaTools"
Set-RegString $uninst "InstallLocation" $InstallDir
Set-RegString $uninst "DisplayIcon" $ExePath
$uninstPs1 = Join-Path $InstallDir "Go-cai-dat-MarkdownTabs-Rust_v0.5.ps1"
Set-RegString $uninst "UninstallString" "powershell.exe -NoProfile -ExecutionPolicy Bypass -File `"$uninstPs1`""
Set-RegDword $uninst "NoModify" 1
Set-RegDword $uninst "NoRepair" 1

# Dat mac dinh
$doDefault = $SetDefault
if (-not $doDefault -and -not $Silent) {
    Say ""
    Say "Windows 10/11 khong cho phan mem tu chiem quyen mac dinh."
    Say "Neu dong y, trinh cai se xoa lua chon mac dinh CU (UserChoice) cho .md/.markdown/.mdown"
    Say "va dat Markdown Tabs lam mac dinh o tang Classes."
    $ans = Read-Host "Dat Markdown Tabs lam mac dinh cho file .md ngay bay gio? [Y/N]"
    $doDefault = ($ans -match '^[Yy]')
}
if ($doDefault) {
    foreach ($ext in $Exts) {
        $ucPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\$ext\UserChoice"
        if (Test-Path $ucPath) { Remove-Item $ucPath -Recurse -Force -ErrorAction SilentlyContinue }
        Set-RegString "HKCU:\Software\Classes\$ext" "(Default)" $ProgId
    }
    Say "Da dat Markdown Tabs lam mac dinh cho .md / .markdown / .mdown."
}

Refresh-Associations
Say ""
Say "=== CAI DAT HOAN TAT ==="
Say "Mo ung dung: Start Menu > Markdown Tabs hoac shortcut Desktop."
Say "Thu ngay: nhap dup file Test-Rust_v0.5.md trong goi nay; khi app dang mo san,"
Say "nhap dup them mot file .md khac ngoai Explorer - file se mo thanh TAB trong cua so do."
if (-not $Silent) { Read-Host "Nhan Enter de dong" }

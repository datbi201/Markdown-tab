# =====================================================================
#  Markdown Tabs (Rust native) - Go cai dat (per-user)
#  Xoa exe, shortcut, dang ky lien ket va muc Uninstall.
#  Giu lai thu muc cau hinh %APPDATA%\MarkdownTabs (phien lam viec).
# =====================================================================
param([switch]$Silent)
$ErrorActionPreference = "Continue"
$ProgId     = "MarkdownTabs.Document"
$Exts       = @(".md", ".markdown", ".mdown")
$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\Markdown Tabs"

function Say($m) { Write-Host $m }
Say "=== Go cai dat Markdown Tabs (Rust) ==="
if (-not $Silent) {
    $ans = Read-Host "Ban chac chan muon go Markdown Tabs? [Y/N]"
    if ($ans -notmatch '^[Yy]') { Say "Da huy."; exit 0 }
}
Get-Process -Name "MarkdownTabs" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 400

$links = @(
    (Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\Markdown Tabs"),
    (Join-Path ([Environment]::GetFolderPath("Desktop")) "Markdown Tabs.lnk")
)
foreach ($l in $links) { if (Test-Path $l) { Remove-Item $l -Recurse -Force -ErrorAction SilentlyContinue } }

Remove-Item "HKCU:\Software\Classes\$ProgId" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item "HKCU:\Software\LisaTools\MarkdownTabs" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item "HKCU:\Software\Classes\Applications\MarkdownTabs.exe" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\MarkdownTabs" -Recurse -Force -ErrorAction SilentlyContinue
$ra = "HKCU:\Software\RegisteredApplications"
if (Test-Path $ra) { Remove-ItemProperty $ra -Name "Markdown Tabs" -ErrorAction SilentlyContinue }
foreach ($ext in $Exts) {
    try {
        $k = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey("Software\Classes\$ext\OpenWithProgids", $true)
        if ($k) { try { $k.DeleteValue($ProgId, $false) } catch { } ; $k.Close() }
    } catch { }
    $cls = "HKCU:\Software\Classes\$ext"
    if (Test-Path $cls) {
        $cur = (Get-Item $cls).GetValue("")
        if ($cur -eq $ProgId) { Set-Item $cls -Value "" }
    }
    $uc = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\$ext\UserChoice"
    if (Test-Path $uc) {
        $p = (Get-ItemProperty $uc -ErrorAction SilentlyContinue).ProgId
        if ($p -eq $ProgId) { Remove-Item $uc -Recurse -Force -ErrorAction SilentlyContinue }
    }
}
try {
    $sig = '[DllImport("shell32.dll")] public static extern void SHChangeNotify(int wEventId, int uFlags, IntPtr dwItem1, IntPtr dwItem2);'
    $t = Add-Type -MemberDefinition $sig -Name ShellNotifyR2 -Namespace Win32 -PassThru
    $t::SHChangeNotify(0x08000000, 0, [IntPtr]::Zero, [IntPtr]::Zero)
} catch { }

if (Test-Path $InstallDir) {
    if ($PSScriptRoot -eq $InstallDir) {
        Start-Process cmd.exe -ArgumentList "/c", "timeout /t 2 /nobreak >nul & rmdir /s /q `"$InstallDir`"" -WindowStyle Hidden
        Say "Da go xong. Thu muc cai dat se tu xoa trong vai giay."
    } else {
        Remove-Item $InstallDir -Recurse -Force -ErrorAction SilentlyContinue
        Say "Da xoa thu muc cai dat."
    }
}
Say "Go cai dat hoan tat."
if (-not $Silent) { Read-Host "Nhan Enter de dong" }

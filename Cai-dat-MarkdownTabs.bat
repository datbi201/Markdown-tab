@echo off
chcp 65001 >nul
rem ============================================================
rem  Markdown Tabs (Rust native) - CAI DAT
rem  Nhap dup chuot de chay. Cai vao
rem  %LOCALAPPDATA%\Programs\Markdown Tabs (khong can admin,
rem  khong can Python). Luon tro toi ban moi nhat.
rem ============================================================
cd /d "%~dp0"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0Cai-dat-MarkdownTabs.ps1" %*
if errorlevel 1 (
    echo.
    echo Cai dat gap loi. Hay chup man hinh thong bao loi gui lai de duoc ho tro.
    pause
)

@echo off
chcp 65001 >nul
rem ============================================================
rem  Markdown Tabs (Rust native) v0.5 - CAI DAT
rem  Nhap dup chuot de chay. Cai vao
rem  %LOCALAPPDATA%\Programs\Markdown Tabs (khong can admin,
rem  khong can Python). Neu da cai ban Python v1.2 thi ban nay
rem  nang cap tai cho (cung ten MarkdownTabs.exe).
rem ============================================================
cd /d "%~dp0"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0Cai-dat-MarkdownTabs-Rust_v0.5.ps1" %*
if errorlevel 1 (
    echo.
    echo Cai dat gap loi. Hay chup man hinh thong bao loi gui lai de duoc ho tro.
    pause
)

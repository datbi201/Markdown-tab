@echo off
chcp 65001 >nul
rem Markdown Tabs (Rust native) - GO CAI DAT
cd /d "%~dp0"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0Go-cai-dat-MarkdownTabs.ps1" %*
if errorlevel 1 pause

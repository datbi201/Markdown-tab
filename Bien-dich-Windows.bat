@echo off
chcp 65001 >nul
echo === Bien dich Markdown Tabs v0.5.0 tren Windows ===
where cargo >nul 2>nul
if errorlevel 1 (
  echo Chua co Rust. Dang tai trinh cai rustup...
  powershell -NoProfile -Command "Invoke-WebRequest https://win.rustup.rs/x86_64 -OutFile $env:TEMP\rustup-init.exe"
  "%TEMP%\rustup-init.exe" -y --default-toolchain stable
  set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
)
cd /d "%~dp0src-rust"
rem Tren Windows dung toolchain MSVC mac dinh, bo cau hinh mingw cua ban cross-compile
if exist .cargo\config.toml ren .cargo\config.toml config.toml.bak
cargo build --release
if errorlevel 1 ( echo LOI BIEN DICH & pause & exit /b 1 )
copy /y target\release\MarkdownTabs.exe "%~dp0MarkdownTabs.exe" >nul
echo.
echo Xong! File MarkdownTabs.exe nam o: %~dp0
echo Chay Cai-dat-MarkdownTabs-Rust_v0.5.ps1 / .bat de cai dat nhu cu.
pause

@echo off
chcp 65001 >nul
setlocal enabledelayedexpansion
for /f "tokens=3 delims= " %%v in ('findstr /r "^version = " "%~dp0src-rust\Cargo.toml"') do set "VER=%%~v"
if "%VER%"=="" set "VER=latest"
echo === Bien dich Markdown Tabs v%VER% tren Windows ===
where cargo >nul 2>nul
if errorlevel 1 (
  if exist "%USERPROFILE%\.cargo\bin\cargo.exe" (
    set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
  ) else (
    echo Chua co Rust. Dang tai trinh cai rustup...
    powershell -NoProfile -Command "Invoke-WebRequest https://win.rustup.rs/x86_64 -OutFile $env:TEMP\rustup-init.exe"
    "%TEMP%\rustup-init.exe" -y --default-toolchain stable
    set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
  )
)
if "%CARGO_TARGET_DIR%"=="" if exist D:\ set "CARGO_TARGET_DIR=D:\mdtabs-target"
cd /d "%~dp0src-rust"
rem Tren Windows dung toolchain MSVC mac dinh, bo cau hinh mingw cua ban cross-compile
if exist .cargo\config.toml ren .cargo\config.toml config.toml.bak
cargo build --release
if errorlevel 1 ( echo LOI BIEN DICH & pause & exit /b 1 )

set "EXE_SRC=%~dp0src-rust\target\release\MarkdownTabs.exe"
if not "%CARGO_TARGET_DIR%"=="" set "EXE_SRC=%CARGO_TARGET_DIR%\release\MarkdownTabs.exe"

if exist "%~dp0..\02_LOCAL_RUNTIME" (
  copy /y "%EXE_SRC%" "%~dp0..\02_LOCAL_RUNTIME\MarkdownTabs.exe" >nul
  echo Da cap nhat: %~dp0..\02_LOCAL_RUNTIME\MarkdownTabs.exe
) else (
  copy /y "%EXE_SRC%" "%~dp0MarkdownTabs.exe" >nul
  echo Da cap nhat: %~dp0MarkdownTabs.exe
)
echo.
echo Xong! Chay Cai-dat-MarkdownTabs.bat de cai dat ban v%VER% moi nhat.
pause

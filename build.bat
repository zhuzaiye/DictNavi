@echo off
REM DictNavi Windows Build Script
REM Used to compile release version of Rust application

echo ========================================
echo DictNavi Windows Build Script
echo ========================================
echo.

REM Check if Rust is installed
where cargo >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo [Error] cargo command not found, please ensure Rust is installed
    echo Visit https://www.rust-lang.org/ to install Rust
    pause
    exit /b 1
)

echo [1/3] Cleaning old build files...
if exist target\release (
    echo Deleting old release directory...
    rmdir /s /q target\release
)

echo.
echo [2/3] Compiling release version (this may take a few minutes)...
cargo build --release
if %ERRORLEVEL% NEQ 0 (
    echo [Error] Compilation failed!
    pause
    exit /b 1
)

echo.
echo [3/3] Build complete!
echo.
echo Executable location: target\release\DictNavi.exe
echo.
echo Next step: Run package.bat to package the application
echo.
pause


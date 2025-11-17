@echo off
REM DictNavi One-click Build and Package Script
REM Automatically executes build and package process

echo ========================================
echo DictNavi One-click Build and Package
echo ========================================
echo.

REM Call build script
call build.bat
if %ERRORLEVEL% NEQ 0 (
    echo [Error] Build failed, stopping package
    pause
    exit /b 1
)

echo.
echo ========================================
echo Starting package...
echo ========================================
echo.

REM Call package script
call package.bat


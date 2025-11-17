@echo off
REM DictNavi Windows Package Script
REM Packages executable and resource files to dist directory

echo ========================================
echo DictNavi Windows Package Script
echo ========================================
echo.

REM Check if executable exists
if not exist target\release\DictNavi.exe (
    echo [Error] Executable file not found: target\release\DictNavi.exe
    echo Please run build.bat first to compile the application
    pause
    exit /b 1
)

REM Check if words directory exists
if not exist words (
    echo [Warning] words directory not found
    echo Application may not work properly
)

REM Create distribution directory
set DIST_DIR=dist\DictNavi
if exist %DIST_DIR% (
    echo Cleaning old distribution directory...
    rmdir /s /q %DIST_DIR%
)

echo [1/4] Creating distribution directory...
mkdir %DIST_DIR%
if %ERRORLEVEL% NEQ 0 (
    echo [Error] Failed to create distribution directory
    pause
    exit /b 1
)

echo [2/4] Copying executable file...
copy target\release\DictNavi.exe %DIST_DIR%\ >nul
if %ERRORLEVEL% NEQ 0 (
    echo [Error] Failed to copy executable file
    pause
    exit /b 1
)

echo [3/4] Copying words directory (including .index)...
if exist words (
    xcopy words %DIST_DIR%\words\ /E /I /H /Y >nul
    if %ERRORLEVEL% NEQ 0 (
        echo [Warning] Issue occurred while copying words directory
    ) else (
        echo      - words directory copied
        if exist words\.index (
            echo      - words\.index directory copied
        )
    )
) else (
    echo [Warning] words directory does not exist, skipping copy
)

echo [4/4] Creating README.txt...
(
echo DictNavi - English Dictionary
echo ==============================
echo.
echo Usage:
echo   1. Double-click DictNavi.exe to start the application
echo   2. words directory contains dictionary data and index
echo   3. To update dictionary, replace JSON files in words directory
echo      and rebuild index within the application
echo.
echo System Requirements:
echo   - Windows 7 or higher
echo   - No additional runtime libraries required
echo.
echo File Structure:
echo   DictNavi.exe      - Main program
echo   words\            - Dictionary data directory
echo     .index\         - Search index (auto-generated)
echo     *.json          - Word definition files
echo.
) > %DIST_DIR%\README.txt

echo.
echo ========================================
echo Package complete!
echo ========================================
echo.
echo Distribution directory: %DIST_DIR%
echo.
echo Directory contents:
dir /B %DIST_DIR%
echo.
echo You can compress the %DIST_DIR% directory into a ZIP file for distribution
echo.
pause


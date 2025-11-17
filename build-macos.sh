#!/bin/bash
# DictNavi macOS Build Script
# Used to compile release version of Rust application for macOS

echo "========================================"
echo "DictNavi macOS Build Script"
echo "========================================"
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "[Error] cargo command not found, please ensure Rust is installed"
    echo "Visit https://www.rust-lang.org/ to install Rust"
    exit 1
fi

echo "[1/3] Cleaning old build files..."
if [ -d "target/release" ]; then
    echo "Deleting old release directory..."
    rm -rf target/release
fi

echo ""
echo "[2/3] Compiling release version for macOS (this may take a few minutes)..."
cargo build --release --target x86_64-apple-darwin
if [ $? -ne 0 ]; then
    echo "[Error] Compilation failed!"
    exit 1
fi

echo ""
echo "[3/3] Build complete!"
echo ""
echo "Executable location: target/x86_64-apple-darwin/release/DictNavi"
echo ""
echo "Next step: Run ./package-macos.sh to package the application"
echo ""


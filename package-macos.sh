#!/bin/bash
# DictNavi macOS Package Script
# Packages executable and resource files to dist directory for macOS

echo "========================================"
echo "DictNavi macOS Package Script"
echo "========================================"
echo ""

# Check if executable exists
EXECUTABLE="target/x86_64-apple-darwin/release/DictNavi"
if [ ! -f "$EXECUTABLE" ]; then
    echo "[Error] Executable file not found: $EXECUTABLE"
    echo "Please run ./build-macos.sh first to compile the application"
    exit 1
fi

# Check if words directory exists
if [ ! -d "words" ]; then
    echo "[Warning] words directory not found"
    echo "Application may not work properly"
fi

# Create distribution directory
DIST_DIR="dist/DictNavi-macOS"
if [ -d "$DIST_DIR" ]; then
    echo "Cleaning old distribution directory..."
    rm -rf "$DIST_DIR"
fi

echo "[1/4] Creating distribution directory..."
mkdir -p "$DIST_DIR"
if [ $? -ne 0 ]; then
    echo "[Error] Failed to create distribution directory"
    exit 1
fi

echo "[2/4] Copying executable file..."
cp "$EXECUTABLE" "$DIST_DIR/DictNavi"
if [ $? -ne 0 ]; then
    echo "[Error] Failed to copy executable file"
    exit 1
fi

# Make executable
chmod +x "$DIST_DIR/DictNavi"

echo "[3/4] Copying words directory (including .index)..."
if [ -d "words" ]; then
    cp -r words "$DIST_DIR/"
    if [ $? -eq 0 ]; then
        echo "     - words directory copied"
        if [ -d "words/.index" ]; then
            echo "     - words/.index directory copied"
        fi
    else
        echo "[Warning] Issue occurred while copying words directory"
    fi
else
    echo "[Warning] words directory does not exist, skipping copy"
fi

echo "[4/4] Creating README.txt..."
cat > "$DIST_DIR/README.txt" << 'EOF'
DictNavi - English Dictionary
=============================

Usage:
  1. Double-click DictNavi to start the application
  2. If macOS shows a security warning, right-click and select "Open"
  3. words directory contains dictionary data and index
  4. To update dictionary, replace JSON files in words directory
     and rebuild index within the application

System Requirements:
  - macOS 10.13 (High Sierra) or higher
  - No additional runtime libraries required

File Structure:
  DictNavi          - Main program
  words/            - Dictionary data directory
    .index/         - Search index (auto-generated)
    *.json          - Word definition files

Note:
  - If you see "DictNavi cannot be opened" error, you may need to:
    1. Right-click DictNavi and select "Open"
    2. Or run: xattr -cr DictNavi
EOF

echo ""
echo "========================================"
echo "Package complete!"
echo "========================================"
echo ""
echo "Distribution directory: $DIST_DIR"
echo ""
echo "Directory contents:"
ls -la "$DIST_DIR"
echo ""
echo "You can compress the $DIST_DIR directory into a ZIP file for distribution"
echo ""


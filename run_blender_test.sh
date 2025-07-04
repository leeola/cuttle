#!/bin/bash

# Automated Blender test runner for Cuttle architecture

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_SCRIPT="$SCRIPT_DIR/test_blender_cli.py"

echo "======================================"
echo "CUTTLE BLENDER AUTOMATED TEST RUNNER"
echo "======================================"

# Check if Blender is available
BLENDER_CMD=""

# First, check if blender is in PATH
if command -v blender >/dev/null 2>&1; then
    BLENDER_CMD="blender"
    echo "Found Blender in PATH"
else
    # Try platform-specific locations
    case "$(uname -s)" in
        Darwin*)
            # macOS locations
            MACOS_LOCATIONS=(
                "/Applications/Blender.app/Contents/MacOS/Blender"
                "/Applications/Blender 4.0/Blender.app/Contents/MacOS/Blender"
                "/Applications/Blender 3.*/Blender.app/Contents/MacOS/Blender"
                "$HOME/Applications/Blender.app/Contents/MacOS/Blender"
            )
            
            for location in "${MACOS_LOCATIONS[@]}"; do
                if [[ -f "$location" ]]; then
                    BLENDER_CMD="$location"
                    echo "Found Blender at: $BLENDER_CMD"
                    break
                fi
            done
            
            if [[ -z "$BLENDER_CMD" ]]; then
                echo "ERROR: Blender not found in PATH or standard macOS locations"
                echo "Searched: ${MACOS_LOCATIONS[*]}"
                echo "Install with: brew install --cask blender"
                echo "Or download from: https://www.blender.org/download/"
                exit 1
            fi
            ;;
            
        Linux*)
            # Linux locations
            LINUX_LOCATIONS=(
                "/usr/bin/blender"
                "/usr/local/bin/blender"
                "/opt/blender/blender"
                "/snap/bin/blender"
                "$HOME/.local/bin/blender"
                "$HOME/blender/blender"
            )
            
            for location in "${LINUX_LOCATIONS[@]}"; do
                if [[ -f "$location" ]]; then
                    BLENDER_CMD="$location"
                    echo "Found Blender at: $BLENDER_CMD"
                    break
                fi
            done
            
            if [[ -z "$BLENDER_CMD" ]]; then
                echo "ERROR: Blender not found in PATH or standard Linux locations"
                echo "Searched: ${LINUX_LOCATIONS[*]}"
                echo "Install with your package manager:"
                echo "  Ubuntu/Debian: sudo apt install blender"
                echo "  Fedora: sudo dnf install blender"
                echo "  Arch: sudo pacman -S blender"
                echo "  Snap: sudo snap install blender --classic"
                echo "Or download from: https://www.blender.org/download/"
                exit 1
            fi
            ;;
            
        *)
            echo "ERROR: Unsupported platform: $(uname -s)"
            echo "Please install Blender and add it to your PATH"
            echo "Or download from: https://www.blender.org/download/"
            exit 1
            ;;
    esac
fi

BLENDER_VERSION=$($BLENDER_CMD --version | head -n 1)
echo "Found: $BLENDER_VERSION"

# Check if test script exists
if [[ ! -f "$TEST_SCRIPT" ]]; then
    echo "ERROR: Test script not found: $TEST_SCRIPT"
    exit 1
fi

echo "Test script: $TEST_SCRIPT"
echo ""

# Run the test
echo "Running Cuttle tests in Blender..."
echo "======================================"

# Run Blender in background mode with our test script
# --background: Run without GUI
# --python: Execute our test script
# --python-exit-code: Exit with Python script's exit code
if "$BLENDER_CMD" --background --python "$TEST_SCRIPT" --python-exit-code 2>&1; then
    echo ""
    echo "======================================"
    echo "PASS: ALL TESTS PASSED!"
    echo "Cuttle architecture is working in Blender"
    echo "Ready for next development phase"
    echo "======================================"
    exit 0
else
    EXIT_CODE=$?
    echo ""
    echo "======================================"
    echo "FAIL: TESTS FAILED!"
    echo "Check the output above for details"
    echo "Exit code: $EXIT_CODE"
    echo "======================================"
    exit $EXIT_CODE
fi
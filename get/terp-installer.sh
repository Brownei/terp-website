#!/bin/bash

# Terp Network Installer
# This script checks for Python 3.6+ and runs the terp-installer.py

set -e

# Detect OS
OS="$(uname -s)"
case "${OS}" in
    Linux*)     MACHINE=Linux;;
    Darwin*)    MACHINE=Mac;;
    *)          MACHINE="UNKNOWN";;
esac

# Check if Python 3 is installed and meets minimum version
check_python() {
    if command -v python3 &> /dev/null; then
        PYTHON_VERSION=$(python3 --version 2>&1 | awk '{print $2}')
        PYTHON_MAJOR=$(echo $PYTHON_VERSION | cut -d. -f1)
        PYTHON_MINOR=$(echo $PYTHON_VERSION | cut -d. -f2)

        if [ "$PYTHON_MAJOR" -ge 3 ] && [ "$PYTHON_MINOR" -ge 6 ]; then
            echo "✓ Python $PYTHON_VERSION found"
            return 0
        else
            echo "✗ Python $PYTHON_VERSION found, but version 3.6+ is required"
            return 1
        fi
    else
        echo "✗ Python 3 is not installed"
        return 1
    fi
}

# Install Python on macOS
install_python_mac() {
    echo "Installing Python on macOS..."

    if ! command -v brew &> /dev/null; then
        echo "Installing Homebrew first..."
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

        # Add Homebrew to PATH for this session
        if [ -f "/opt/homebrew/bin/brew" ]; then
            eval "$(/opt/homebrew/bin/brew shellenv)"
        elif [ -f "/usr/local/bin/brew" ]; then
            eval "$(/usr/local/bin/brew shellenv)"
        fi
    fi

    brew install python3
    echo "✓ Python installed successfully"
}

# Install Python on Linux
install_python_linux() {
    echo "Installing Python on Linux..."

    if command -v apt-get &> /dev/null; then
        sudo apt-get update
        sudo apt-get install -y python3 python3-pip
    elif command -v yum &> /dev/null; then
        sudo yum install -y python3 python3-pip
    elif command -v dnf &> /dev/null; then
        sudo dnf install -y python3 python3-pip
    elif command -v pacman &> /dev/null; then
        sudo pacman -S --noconfirm python python-pip
    else
        echo "Could not detect package manager."
        echo "Please install Python 3.6+ manually: https://www.python.org/downloads/"
        exit 1
    fi

    echo "✓ Python installed successfully"
}

# Main installation flow
echo "Checking Python installation..."

if ! check_python; then
    echo ""
    echo "Python 3.6+ is required to run the Terp Network installer."
    read -p "Would you like to install Python now? (y/n) " -n 1 -r
    echo

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        if [ "$MACHINE" = "Mac" ]; then
            install_python_mac
        elif [ "$MACHINE" = "Linux" ]; then
            install_python_linux
        else
            echo "Unsupported OS. Please install Python manually."
            exit 1
        fi

        # Verify installation
        if ! check_python; then
            echo "Python installation failed. Please install manually and try again."
            exit 1
        fi
    else
        echo "Installation cancelled. Please install Python 3.6+ and try again."
        echo "Visit: https://www.python.org/downloads/"
        exit 1
    fi
fi

echo ""
echo "Starting Terp Network installer..."
echo ""

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PYTHON_INSTALLER="$SCRIPT_DIR/terp-installer.py"

# Check if we have the Python installer locally
if [ -f "$PYTHON_INSTALLER" ]; then
    # Run local installer
    python3 "$PYTHON_INSTALLER" "$@"
else
    # Download and run from terp.network/get
    echo "Downloading installer from terp.network/get..."
    curl -sL https://terp.network/get > /tmp/terp-installer.py
    python3 /tmp/terp-installer.py "$@"
    rm -f /tmp/terp-installer.py
fi

# Source profile if it exists
if [ -f ~/.profile ]; then
    source ~/.profile
fi

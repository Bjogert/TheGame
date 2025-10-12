#!/bin/bash

# LinusRunThisToSetItUp.sh
# Quick setup script for TheGame Docker environment on Linux
# Assumes Docker, Git, and repo are already available
# Run with: bash LinusRunThisToSetItUp.sh

set -e  # Exit on any error

echo "🐳 Checking TheGame Docker Setup on Linux..."
echo "============================================="

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -f "docker-compose.yml" ]; then
    echo "❌ Not in TheGame directory!"
    echo "   Please run this script from the TheGame folder"
    echo "   cd TheGame && bash LinusRunThisToSetItUp.sh"
    exit 1
fi

echo "✅ Found TheGame project files"

# Check Docker
if ! command_exists docker; then
    echo "❌ Docker not found! Please install Docker first."
    exit 1
fi

echo "✅ Docker found: $(docker --version)"

# Check if Docker daemon is running
if ! docker info >/dev/null 2>&1; then
    echo "🔄 Starting Docker service..."
    sudo systemctl start docker || {
        echo "❌ Failed to start Docker. Please start it manually:"
        echo "   sudo systemctl start docker"
        exit 1
    }
fi

echo "✅ Docker daemon is running"

# Check if user is in docker group
if ! groups $USER | grep -q docker; then
    echo "👤 Adding user to docker group..."
    sudo usermod -aG docker $USER
    echo "⚠️  You'll need to log out and back in for group changes to take effect"
    echo "   Or run: newgrp docker"
fi

# Check for X11/Wayland for graphics
if [ -z "$DISPLAY" ] && [ -z "$WAYLAND_DISPLAY" ]; then
    echo "⚠️  No display detected. Graphics may not work."
    echo "   Make sure you're running this from a desktop session"
else
    echo "✅ Display detected: DISPLAY=${DISPLAY:-none} WAYLAND_DISPLAY=${WAYLAND_DISPLAY:-none}"
fi

# Check for xhost (needed for X11 forwarding)
if command_exists xhost; then
    echo "�️  Setting up X11 forwarding..."
    xhost +local: >/dev/null 2>&1 || echo "⚠️  X11 forwarding setup failed (this might be OK)"
    echo "✅ X11 tools available"
else
    echo "�📦 Installing X11 utilities..."
    if command_exists apt-get; then
        sudo apt update && sudo apt install -y x11-xserver-utils
    elif command_exists dnf; then
        sudo dnf install -y xorg-x11-server-utils
    elif command_exists pacman; then
        sudo pacman -S --needed --noconfirm xorg-xhost
    else
        echo "⚠️  Could not install xhost. Graphics forwarding might not work."
    fi
fi

# Check for necessary group permissions (for /dev/dri access)
if [ -d "/dev/dri" ]; then
    if groups $USER | grep -q "video\|render"; then
        echo "✅ User has video/render group access"
    else
        echo "� Adding user to video group for GPU access..."
        sudo usermod -aG video $USER 2>/dev/null || true
        # Try render group too (newer systems)
        sudo usermod -aG render $USER 2>/dev/null || true
    fi
else
    echo "⚠️  No GPU devices found at /dev/dri"
fi

# Test Docker Compose
if ! command_exists docker-compose && ! docker compose version >/dev/null 2>&1; then
    echo "❌ Docker Compose not found!"
    echo "   Please install docker-compose or update Docker to include compose plugin"
    exit 1
fi

echo "✅ Docker Compose available"

# Build the Docker image
echo "🔨 Building Docker image (this may take a while on first run)..."
docker compose build

# Test that everything works
echo "🧪 Testing Docker build..."
docker compose run --rm thegame cargo check

echo ""
echo "� Everything looks good! Ready to run the Bevy game."
echo ""
echo "To start the game:"
echo "   docker-compose up"
echo ""
echo "To rebuild and run (if you've made changes):"
echo "   docker-compose up --build"
echo ""
echo "📝 Notes:"
echo "• The game window should appear on your desktop"
echo "• Graphics are forwarded through X11/Wayland"
echo "• If graphics don't work, try: xhost +local:docker"
echo ""
echo "🚀 Want to run it now? (y/n)"
read -p ">> " run_now
if [[ $run_now =~ ^[Yy]$ ]]; then
    echo "🎮 Starting the game..."
    docker-compose up --build
fi
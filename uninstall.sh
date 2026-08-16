#!/usr/bin/env bash
set -e

# LLM-TUI Uninstallation Script
# Version: 1.0.2
# Repository: https://github.com/klt-mm/llm-tui

BINARY_NAME="llm-tui"
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="$HOME/.config/llm-tui"
DATA_DIR="$HOME"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print functions
info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Confirm uninstallation
confirm() {
    echo ""
    echo "╔════════════════════════════════════════╗"
    echo "║   LLM-TUI Uninstallation Script        ║"
    echo "╚════════════════════════════════════════╝"
    echo ""
    echo "This will remove:"
    echo "  - Binary: $INSTALL_DIR/$BINARY_NAME"
    echo "  - Configuration: $CONFIG_DIR"
    echo "  - Database files: $DATA_DIR/llm-tui.db"
    echo ""
    read -p "Are you sure you want to continue? (y/N) " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        info "Uninstallation cancelled"
        exit 0
    fi
}

# Remove binary
remove_binary() {
    local binary_path="$INSTALL_DIR/$BINARY_NAME"
    
    if [[ -f "$binary_path" ]]; then
        info "Removing binary from $binary_path..."
        if [[ ! -w "$INSTALL_DIR" ]]; then
            sudo rm -f "$binary_path"
        else
            rm -f "$binary_path"
        fi
        success "Binary removed"
    else
        warn "Binary not found at $binary_path"
    fi
}

# Remove configuration
remove_config() {
    if [[ -d "$CONFIG_DIR" ]]; then
        info "Removing configuration directory..."
        rm -rf "$CONFIG_DIR"
        success "Configuration removed"
    else
        warn "Configuration directory not found"
    fi
}

# Remove database files
remove_database() {
    local db_files=("$DATA_DIR/llm-tui.db" "$DATA_DIR/llm-tui.db-shm" "$DATA_DIR/llm-tui.db-wal")
    local found=false
    
    for db_file in "${db_files[@]}"; do
        if [[ -f "$db_file" ]]; then
            info "Removing database file: $db_file"
            rm -f "$db_file"
            found=true
        fi
    done
    
    if [[ "$found" == true ]]; then
        success "Database files removed"
    else
        warn "No database files found"
    fi
}

# Remove from PATH if in custom location
remove_from_path() {
    local custom_locations=(
        "$HOME/.local/bin/$BINARY_NAME"
        "$HOME/.cargo/bin/$BINARY_NAME"
    )
    
    for location in "${custom_locations[@]}"; do
        if [[ -f "$location" ]]; then
            info "Removing binary from $location..."
            rm -f "$location"
            success "Removed from $location"
        fi
    done
}

# Main uninstallation function
main() {
    confirm
    
    echo ""
    info "Starting uninstallation..."
    echo ""
    
    # Remove binary
    remove_binary
    
    # Remove from custom locations
    remove_from_path
    
    # Remove configuration
    remove_config
    
    # Remove database
    remove_database
    
    echo ""
    success "╔════════════════════════════════════════╗"
    success "║   Uninstallation Complete!             ║"
    success "╚════════════════════════════════════════╝"
    echo ""
    info "LLM-TUI has been successfully uninstalled"
    info "Thank you for using LLM-TUI!"
    echo ""
    info "To reinstall, run:"
    info "  curl -fsSL https://raw.githubusercontent.com/klt-mm/llm-tui/main/install.sh | bash"
    echo ""
}

# Run main function
main "$@"

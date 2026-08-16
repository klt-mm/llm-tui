#!/usr/bin/env bash
set -e

# LLM-TUI Installation Script
# Version: 1.0.2
# Repository: https://github.com/klt-mm/llm-tui

REPO_URL="https://github.com/klt-mm/llm-tui"
BINARY_NAME="llm-tui"
INSTALL_DIR="/usr/local/bin"
VERSION="1.0.2"

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

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Detect OS
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo "linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    elif [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
        echo "windows"
    else
        error "Unsupported operating system: $OSTYPE"
    fi
}

# Detect architecture
detect_arch() {
    local arch=$(uname -m)
    case "$arch" in
        x86_64|amd64)
            echo "x86_64"
            ;;
        aarch64|arm64)
            echo "aarch64"
            ;;
        *)
            error "Unsupported architecture: $arch"
            ;;
    esac
}

# Install dependencies
install_dependencies() {
    local os=$1
    info "Installing dependencies for $os..."
    
    case "$os" in
        linux)
            if command_exists apt-get; then
                sudo apt-get update
                sudo apt-get install -y build-essential pkg-config libssl-dev git sqlite3 libsqlite3-dev
            elif command_exists dnf; then
                sudo dnf install -y gcc gcc-c++ make pkgconf openssl-devel git sqlite sqlite-devel
            elif command_exists pacman; then
                sudo pacman -S --noconfirm base-devel pkgconf openssl git sqlite
            else
                warn "Could not detect package manager. Please install dependencies manually."
            fi
            ;;
        macos)
            if ! command_exists brew; then
                error "Homebrew is required. Install from https://brew.sh"
            fi
            brew install rust sqlite
            ;;
        windows)
            warn "Windows installation requires manual setup."
            warn "Please install Rust from https://rustup.rs and build from source."
            exit 1
            ;;
    esac
}

# Install Rust if not present
install_rust() {
    if ! command_exists rustc; then
        info "Rust not found. Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
        success "Rust installed successfully"
    else
        info "Rust is already installed: $(rustc --version)"
    fi
}

# Build from source
build_from_source() {
    local temp_dir=$(mktemp -d)
    cd "$temp_dir"
    
    info "Cloning repository..."
    git clone "$REPO_URL"
    cd llm-tui
    
    info "Building LLM-TUI in release mode..."
    cargo build --release
    
    local binary_path="./target/release/$BINARY_NAME"
    
    if [[ ! -f "$binary_path" ]]; then
        error "Build failed: binary not found at $binary_path"
    fi
    
    success "Build completed successfully"
    echo "$temp_dir/$binary_path"
}

# Install binary
install_binary() {
    local binary_path=$1
    
    info "Installing $BINARY_NAME to $INSTALL_DIR..."
    
    if [[ ! -w "$INSTALL_DIR" ]]; then
        sudo mkdir -p "$INSTALL_DIR"
        sudo cp "$binary_path" "$INSTALL_DIR/$BINARY_NAME"
        sudo chmod +x "$INSTALL_DIR/$BINARY_NAME"
    else
        mkdir -p "$INSTALL_DIR"
        cp "$binary_path" "$INSTALL_DIR/$BINARY_NAME"
        chmod +x "$INSTALL_DIR/$BINARY_NAME"
    fi
    
    success "Binary installed to $INSTALL_DIR/$BINARY_NAME"
}

# Create config directory
create_config_dir() {
    local config_dir="$HOME/.config/llm-tui"
    
    if [[ ! -d "$config_dir" ]]; then
        info "Creating configuration directory..."
        mkdir -p "$config_dir"
        
        # Create default config
        cat > "$config_dir/config.toml" << 'EOF'
# LLM-TUI Configuration
# See documentation for more options

[provider]
# base_url = "http://localhost:8080/v1"
# api_key = "your-api-key-here"

[generation]
temperature = 0.7
top_p = 0.9
max_tokens = 2048

[context]
max_tokens = 4096
reserve_for_response = 1024
EOF
        
        success "Configuration directory created at $config_dir"
    fi
}

# Cleanup
cleanup() {
    local temp_dir=$1
    if [[ -n "$temp_dir" ]] && [[ -d "$temp_dir" ]]; then
        info "Cleaning up temporary files..."
        rm -rf "$temp_dir"
    fi
}

# Main installation function
main() {
    echo ""
    echo "╔════════════════════════════════════════╗"
    echo "║   LLM-TUI Installation Script v$VERSION  ║"
    echo "╚════════════════════════════════════════╝"
    echo ""
    
    local os=$(detect_os)
    local arch=$(detect_arch)
    
    info "Detected OS: $os"
    info "Detected Architecture: $arch"
    echo ""
    
    # Install dependencies
    install_dependencies "$os"
    
    # Install Rust
    install_rust
    
    # Build from source
    local binary_path
    binary_path=$(build_from_source)
    local temp_dir=$(dirname $(dirname $(dirname "$binary_path")))
    
    # Install binary
    install_binary "$binary_path"
    
    # Create config directory
    create_config_dir
    
    # Cleanup
    cleanup "$temp_dir"
    
    echo ""
    success "╔════════════════════════════════════════╗"
    success "║   Installation Complete!               ║"
    success "╚════════════════════════════════════════╝"
    echo ""
    info "Run 'llm-tui' to start the application"
    info "Configuration file: ~/.config/llm-tui/config.toml"
    info "Documentation: https://klt-mm.github.io/llm-tui"
    echo ""
    info "To uninstall, run:"
    info "  curl -fsSL https://raw.githubusercontent.com/klt-mm/llm-tui/main/uninstall.sh | bash"
    echo ""
}

# Run main function
main "$@"

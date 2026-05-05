#!/usr/bin/env bash

set -e

VERSION="0.3.2"
REPO="thepinak503/echomind"
BINARY_NAME="echomind"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Detect OS
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo "linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    else
        error "Unsupported operating system: $OSTYPE"
        exit 1
    fi
}

# Detect architecture
detect_arch() {
    local arch=$(uname -m)
    case "$arch" in
        x86_64|amd64)
            echo "amd64"
            ;;
        arm64|aarch64)
            echo "arm64"
            ;;
        *)
            error "Unsupported architecture: $arch"
            exit 1
            ;;
    esac
}

# Check for required commands
check_dependencies() {
    local deps=("curl" "tar")
    
    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &> /dev/null; then
            error "Required command not found: $dep"
            exit 1
        fi
    done
}

# Download binary
download_binary() {
    local os=$1
    local arch=$2
    
    info "Downloading echomind for $os-$arch..."
    
    local download_url
    local archive_name
    
    if [[ "$os" == "linux" ]]; then
        download_url="https://github.com/${REPO}/releases/download/v${VERSION}/echomind-linux-${arch}.tar.gz"
        archive_name="echomind-linux-${arch}.tar.gz"
    elif [[ "$os" == "macos" ]]; then
        download_url="https://github.com/${REPO}/releases/download/v${VERSION}/echomind-macos-${arch}.tar.gz"
        archive_name="echomind-macos-${arch}.tar.gz"
    fi
    
    info "Downloading from: $download_url"
    
    if ! curl -fsSL -o "/tmp/${archive_name}" "$download_url"; then
        error "Failed to download binary"
        exit 1
    fi
    
    info "Extracting archive..."
    cd /tmp
    tar -xzf "${archive_name}"
    
    info "Installing binary..."
    local install_dir="/usr/local/bin"
    if [[ ! -w "$install_dir" ]]; then
        install_dir="$HOME/.local/bin"
        mkdir -p "$install_dir"
        warning "No write permission to /usr/local/bin, installing to $install_dir"
        warning "Make sure $install_dir is in your PATH"
    fi
    
    if [[ "$os" == "macos" ]]; then
        # macOS might need to remove quarantine attribute
        xattr -d "$BINARY_NAME" 2>/dev/null || true
    fi
    
    if install -m 755 "$BINARY_NAME" "$install_dir/$BINARY_NAME"; then
        success "Binary installed to $install_dir/$BINARY_NAME"
    else
        error "Failed to install binary"
        exit 1
    fi
    
    # Cleanup
    rm -f "/tmp/${archive_name}"
    rm -f "/tmp/$BINARY_NAME"
}

# Install shell completions
install_completions() {
    local os=$1
    
    info "Installing shell completions..."
    
    if [[ "$os" == "macos" ]]; then
        local bash_completion="$HOME/.bash_completion"
        local zsh_completion="$HOME/.zsh/completions"
        
        # Create directories if they don't exist
        mkdir -p "$(dirname "$bash_completion")" 2>/dev/null || true
        mkdir -p "$(dirname "$zsh_completion")" 2>/dev/null || true
        
        # Download and install completions
        if curl -fsSL -o "/tmp/echomind.bash" "https://raw.githubusercontent.com/${REPO}/master/docs/completions/echomind.bash"; then
            cp "/tmp/echomind.bash" "$bash_completion/"
            success "Bash completion installed"
        fi
        
        if curl -fsSL -o "/tmp/_echomind" "https://raw.githubusercontent.com/${REPO}/master/docs/completions/_echomind"; then
            cp "/tmp/_echomind" "$zsh_completion/"
            success "Zsh completion installed"
        fi
        
        rm -f "/tmp/echomind.bash" "/tmp/_echomind"
    elif [[ "$os" == "linux" ]]; then
        local bash_completion="/usr/share/bash-completion/completions"
        local zsh_completion="/usr/share/zsh/site-functions"
        local fish_completion="/usr/share/fish/vendor_completions.d"
        
        # Download completions
        if curl -fsSL -o "/tmp/echomind.bash" "https://raw.githubusercontent.com/${REPO}/master/docs/completions/echomind.bash"; then
            sudo install -m 644 "/tmp/echomind.bash" "$bash_completion/echomind" 2>/dev/null || true
            success "Bash completion installed"
        fi
        
        if curl -fsSL -o "/tmp/_echomind" "https://raw.githubusercontent.com/${REPO}/master/docs/completions/_echomind"; then
            sudo install -m 644 "/tmp/_echomind" "$zsh_completion/" 2>/dev/null || true
            success "Zsh completion installed"
        fi
        
        if curl -fsSL -o "/tmp/echomind.fish" "https://raw.githubusercontent.com/${REPO}/master/docs/completions/echomind.fish"; then
            sudo install -m 644 "/tmp/echomind.fish" "$fish_completion/" 2>/dev/null || true
            success "Fish completion installed"
        fi
        
        rm -f "/tmp/echomind.bash" "/tmp/_echomind" "/tmp/echomind.fish"
    fi
}

# Install via package manager (Linux only)
install_via_package_manager() {
    local os=$1
    
    if [[ "$os" == "linux" ]]; then
        if command -v apt-get &> /dev/null; then
            info "Installing via apt (Debian/Ubuntu)..."
            
            # Check for yay (AUR helper)
            if command -v yay &> /dev/null; then
                info "Installing via yay (AUR)..."
                yay -S echomind
                return
            fi
            
            # Check for pacman
            if command -v pacman &> /dev/null; then
                info "Installing via pacman (Arch Linux)..."
                pacman -S echomind 2>/dev/null || {
                    error "echomind not found in pacman, trying yay..."
                    if command -v yay &> /dev/null; then
                        yay -S echomind
                        return
                    fi
                }
                return
            fi
            
            # Check for dpkg (Debian)
            if command -v dpkg &> /dev/null; then
                info "Installing via dpkg (Debian/Ubuntu)..."
                info "Downloading .deb package..."
                local deb_url="https://github.com/${REPO}/releases/download/v${VERSION}/echomind_0.3.2.1_amd64.deb"
                if curl -fsSL -o "/tmp/echomind.deb" "$deb_url"; then
                    info "Installing..."
                    sudo dpkg -i "/tmp/echomind.deb"
                    success "Debian package installed!"
                    rm -f "/tmp/echomind.deb"
                    return
                else
                    warning "Failed to download .deb, trying cargo..."
                fi
            fi
            
            # Check for cargo (Rust package manager)
            if command -v cargo &> /dev/null; then
                info "Installing via cargo..."
                cargo install echomind
                return
            fi
            
            # Fallback to binary installation
            warning "No package manager found, installing from binary..."
            return 1
        fi
    fi
    
    return 1
}

# Verify installation
verify_installation() {
    if command -v echomind &> /dev/null; then
        local version=$(echomind --version 2>/dev/null || echo "unknown")
        success "echomind installed successfully! Version: $version"
        info "Run 'echomind --help' for usage information"
    else
        error "Installation verification failed"
        exit 1
    fi
}

# Display installation instructions
display_instructions() {
    local install_dir=$1
    
    echo ""
    echo "=========================================="
    echo "Installation complete!"
    echo "=========================================="
    echo ""
    
    if [[ "$install_dir" == "$HOME/.local/bin" ]]; then
        echo "IMPORTANT: Add $install_dir to your PATH"
        echo ""
        echo "For Bash, add to ~/.bashrc:"
        echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
        echo ""
        echo "For Zsh, add to ~/.zshrc:"
        echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
        echo ""
        echo "Then run: source ~/.bashrc (or source ~/.zshrc)"
    fi
    
    echo ""
    echo "Quick start:"
    echo "  echomind --help"
    echo "  echomind 'Hello, how are you?'"
    echo ""
}

# Main installation
main() {
    info "Starting echomind installation..."
    echo ""
    
    # Check dependencies
    check_dependencies
    
    # Detect platform
    local os=$(detect_os)
    info "Detected OS: $os"
    
    local arch=$(detect_arch)
    info "Detected architecture: $arch"
    echo ""
    
    # Try package manager first
    if ! install_via_package_manager "$os"; then
        # Fall back to binary installation
        download_binary "$os" "$arch"
        install_completions "$os"
    fi
    
    # Verify installation
    verify_installation
    
    # Display instructions
    local install_dir="/usr/local/bin"
    if [[ ! -w "/usr/local/bin" ]]; then
        install_dir="$HOME/.local/bin"
    fi
    display_instructions "$install_dir"
    
    echo "Installation completed successfully!"
}

# Allow script to be sourced or piped
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
else
    # Script is being sourced, export functions
    export -f install_echomind
    install_echomind() {
        main "$@"
    }
fi

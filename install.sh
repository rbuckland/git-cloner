#!/usr/bin/env bash

# git-cloner installer script
# Usage: curl -fsSL https://rbuckland.github.io/git-cloner/install.sh | bash

set -e

REPO="rbuckland/git-cloner"
INSTALL_DIR="${HOME}/.local/bin"
BINARY_NAME="git-cloner"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Detect OS and architecture
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"
    
    case "$OS" in
        Linux*)     OS="linux";;
        Darwin*)    OS="darwin";;
        *)          error "Unsupported operating system: $OS";;
    esac
    
    case "$ARCH" in
        x86_64)     ARCH="x86_64";;
        aarch64|arm64) ARCH="aarch64";;
        *)          error "Unsupported architecture: $ARCH";;
    esac
    
    info "Detected platform: $OS-$ARCH"
}

# Check if cargo is available
has_cargo() {
    command -v cargo >/dev/null 2>&1
}

# Install from source using cargo
install_from_source() {
    info "Installing from source using cargo..."
    
    if ! has_cargo; then
        error "Cargo not found. Please install Rust from https://rustup.rs/ or download a pre-built binary."
    fi
    
    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"
    
    info "Cloning repository..."
    git clone "https://github.com/${REPO}.git" || error "Failed to clone repository"
    cd git-cloner
    
    info "Building git-cloner (this may take a few minutes)..."
    cargo build --release || error "Failed to build git-cloner"
    
    mkdir -p "$INSTALL_DIR"
    cp "target/release/$BINARY_NAME" "$INSTALL_DIR/" || error "Failed to copy binary"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"
    
    # Copy completion scripts
    mkdir -p "${HOME}/.local/share/git-cloner"
    cp support/git-cloner-completion.bash "${HOME}/.local/share/git-cloner/" 2>/dev/null || true
    cp support/git-cloner-completion.zsh "${HOME}/.local/share/git-cloner/" 2>/dev/null || true
    
    cd /
    rm -rf "$TEMP_DIR"
    
    info "Successfully installed from source!"
}

# Try to download pre-built binary from GitHub releases
install_from_release() {
    info "Attempting to download pre-built binary..."
    
    # Get latest release info
    RELEASE_URL="https://api.github.com/repos/${REPO}/releases/latest"
    DOWNLOAD_URL=$(curl -fsSL "$RELEASE_URL" | grep "browser_download_url.*${OS}.*${ARCH}" | cut -d '"' -f 4 | head -n 1)
    
    if [ -z "$DOWNLOAD_URL" ]; then
        warn "No pre-built binary found for $OS-$ARCH"
        return 1
    fi
    
    info "Downloading from: $DOWNLOAD_URL"
    TEMP_FILE=$(mktemp)
    
    if curl -fsSL -o "$TEMP_FILE" "$DOWNLOAD_URL"; then
        mkdir -p "$INSTALL_DIR"
        
        # Handle different archive formats
        if [[ "$DOWNLOAD_URL" == *.tar.gz ]]; then
            tar -xzf "$TEMP_FILE" -C "$INSTALL_DIR" "$BINARY_NAME"
        elif [[ "$DOWNLOAD_URL" == *.zip ]]; then
            unzip -q "$TEMP_FILE" "$BINARY_NAME" -d "$INSTALL_DIR"
        else
            # Assume it's a raw binary
            mv "$TEMP_FILE" "$INSTALL_DIR/$BINARY_NAME"
        fi
        
        chmod +x "$INSTALL_DIR/$BINARY_NAME"
        rm -f "$TEMP_FILE"
        
        # Download completion scripts
        mkdir -p "${HOME}/.local/share/git-cloner"
        info "Downloading completion scripts..."
        curl -fsSL "https://raw.githubusercontent.com/${REPO}/main/support/git-cloner-completion.bash" -o "${HOME}/.local/share/git-cloner/git-cloner-completion.bash" 2>/dev/null || warn "Failed to download bash completion"
        curl -fsSL "https://raw.githubusercontent.com/${REPO}/main/support/git-cloner-completion.zsh" -o "${HOME}/.local/share/git-cloner/git-cloner-completion.zsh" 2>/dev/null || warn "Failed to download zsh completion"
        
        info "Successfully installed pre-built binary!"
        return 0
    else
        warn "Failed to download pre-built binary"
        return 1
    fi
}

# Setup git alias
setup_git_alias() {
    info "Setting up git alias..."
    
    # Check if alias already exists
    if git config --global alias.cloner >/dev/null 2>&1; then
        EXISTING_ALIAS=$(git config --global alias.cloner)
        if [[ "$EXISTING_ALIAS" == *"git-cloner"* ]]; then
            info "Git alias 'cloner' already configured"
            return
        else
            warn "Git alias 'cloner' exists but points to: $EXISTING_ALIAS"
            read -p "Overwrite? (y/N) " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                info "Skipping git alias configuration"
                return
            fi
        fi
    fi
    
    git config --global alias.cloner "!${INSTALL_DIR}/${BINARY_NAME}" || error "Failed to set git alias"
    info "Git alias configured: 'git cloner' is now available"
}

# Setup shell completion
setup_completion() {
    info "Setting up shell completion..."
    
    COMPLETION_DIR="${HOME}/.local/share/git-cloner"
    
    # Detect user's shell
    USER_SHELL="$(basename "$SHELL")"
    
    case "$USER_SHELL" in
        bash)
            BASHRC="${HOME}/.bashrc"
            SOURCE_LINE="source ${COMPLETION_DIR}/git-cloner-completion.bash"
            
            if [ -f "$BASHRC" ] && grep -q "$SOURCE_LINE" "$BASHRC" 2>/dev/null; then
                info "Bash completion already configured"
            else
                echo "" >> "$BASHRC"
                echo "# git-cloner completion" >> "$BASHRC"
                echo "$SOURCE_LINE" >> "$BASHRC"
                info "Bash completion configured in $BASHRC"
                info "Run 'source ~/.bashrc' or restart your shell to enable completion"
            fi
            ;;
        zsh)
            ZSHRC="${HOME}/.zshrc"
            SOURCE_LINE="source ${COMPLETION_DIR}/git-cloner-completion.zsh"
            
            if [ -f "$ZSHRC" ] && grep -q "$SOURCE_LINE" "$ZSHRC" 2>/dev/null; then
                info "Zsh completion already configured"
            else
                echo "" >> "$ZSHRC"
                echo "# git-cloner completion" >> "$ZSHRC"
                echo "$SOURCE_LINE" >> "$ZSHRC"
                info "Zsh completion configured in $ZSHRC"
                info "Run 'source ~/.zshrc' or restart your shell to enable completion"
            fi
            ;;
        *)
            warn "Shell completion for $USER_SHELL not supported. Supported shells: bash, zsh"
            ;;
    esac
}

# Check PATH
check_path() {
    if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
        warn "${INSTALL_DIR} is not in your PATH"
        info "Add this to your shell configuration file (~/.bashrc, ~/.zshrc, etc.):"
        echo ""
        echo "    export PATH=\"\$PATH:${INSTALL_DIR}\""
        echo ""
    fi
}

# Main installation flow
main() {
    info "Installing git-cloner..."
    
    detect_platform
    
    # Try release first, fall back to source
    if ! install_from_release; then
        install_from_source
    fi
    
    # Verify installation
    if [ ! -f "$INSTALL_DIR/$BINARY_NAME" ]; then
        error "Installation failed: binary not found at $INSTALL_DIR/$BINARY_NAME"
    fi
    
    info "Binary installed to: $INSTALL_DIR/$BINARY_NAME"
    
    # Setup git alias and completion
    setup_git_alias
    setup_completion
    check_path
    
    echo ""
    info "Installation complete! 🎉"
    echo ""
    echo "Usage:"
    echo "  git cloner <repo-name>           # When in a projects/github.com/<org>/ directory"
    echo "  git cloner <org>/<repo>          # Clone from specific org"
    echo "  git cloner <full-url>            # Clone from full URL"
    echo ""
    echo "Examples:"
    echo "  git cloner my-repo"
    echo "  git cloner torvalds/linux"
    echo "  git cloner https://github.com/rust-lang/rust"
    echo ""
}

main "$@"

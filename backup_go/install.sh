#!/bin/bash
set -euo pipefail

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║  🧠 CYNAPSE Installation               ║${NC}"
echo -e "${GREEN}║  Modular AI Agent System               ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
echo ""

# Detect OS and architecture
detect_os() {
    KERNEL=$(uname -s)
    ARCH=$(uname -m)
    
    case "$KERNEL" in
        Linux*)
            OS_TYPE="linux"
            OS_NAME="Linux"
            case "$ARCH" in
                x86_64)
                    OS_ARCH="amd64"
                    ;;
                aarch64)
                    OS_ARCH="arm64"
                    ;;
                armv7l)
                    OS_ARCH="armv7"
                    ;;
                *)
                    echo -e "${RED}Unsupported architecture: $ARCH${NC}"
                    exit 1
                    ;;
            esac
            ;;
        Darwin*)
            OS_TYPE="darwin"
            OS_NAME="macOS"
            case "$ARCH" in
                x86_64)
                    OS_ARCH="amd64"
                    ;;
                arm64)
                    OS_ARCH="arm64"
                    ;;
                *)
                    echo -e "${RED}Unsupported architecture: $ARCH${NC}"
                    exit 1
                    ;;
            esac
            ;;
        MINGW*|MSYS*|CYGWIN*)
            OS_TYPE="windows"
            OS_NAME="Windows"
            OS_ARCH="amd64"
            ;;
        *)
            echo -e "${RED}Unsupported OS: $KERNEL${NC}"
            exit 1
            ;;
    esac
}

# Install dependencies based on OS
install_dependencies() {
    if [ "$OS_TYPE" = "linux" ]; then
        if command -v apt &> /dev/null; then
            echo "Using apt package manager..."
            sudo apt-get update -qq
            sudo apt-get install -y -qq \
                build-essential \
                pkg-config \
                libopenblas-dev \
                libssl-dev \
                libsqlite3-dev \
                git \
                curl
        elif command -v yum &> /dev/null; then
            echo "Using yum package manager..."
            sudo yum groupinstall -y "Development Tools"
            sudo yum install -y \
                pkgconfig \
                openblas-devel \
                openssl-devel \
                sqlite-devel \
                git \
                curl
        elif command -v pacman &> /dev/null; then
            echo "Using pacman package manager..."
            sudo pacman -S --noconfirm \
                base-devel \
                pkg-config \
                openblas \
                openssl \
                sqlite \
                git \
                curl
        else
            echo -e "${YELLOW}Could not detect package manager. Please install dependencies manually.${NC}"
        fi
    elif [ "$OS_TYPE" = "darwin" ]; then
        if ! command -v brew &> /dev/null; then
            echo "Installing Homebrew..."
            /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
        fi
        echo "Installing dependencies with Homebrew..."
        brew install openblas pkg-config sqlite3
    fi
}

# Install Go if not present
install_go() {
    if command -v go &> /dev/null; then
        GO_VERSION=$(go version | awk '{print $3}' | sed 's/go//')
        echo -e "${GREEN}✓ Go $GO_VERSION already installed${NC}"
        return
    fi
    
    echo "Installing Go..."
    GO_VERSION="1.22.0"
    
    if [ "$OS_TYPE" = "linux" ]; then
        curl -fsSL https://go.dev/dl/go${GO_VERSION}.${OS_TYPE}-${OS_ARCH}.tar.gz -o /tmp/go.tar.gz
        sudo rm -rf /usr/local/go
        sudo tar -C /usr/local -xzf /tmp/go.tar.gz
        rm /tmp/go.tar.gz
        
        # Add to PATH
        if ! grep -q "/usr/local/go/bin" ~/.bashrc; then
            echo 'export PATH=$PATH:/usr/local/go/bin' >> ~/.bashrc
        fi
        export PATH=$PATH:/usr/local/go/bin
        
    elif [ "$OS_TYPE" = "darwin" ]; then
        curl -fsSL https://go.dev/dl/go${GO_VERSION}.${OS_TYPE}-${OS_ARCH}.pkg -o /tmp/go.pkg
        sudo installer -pkg /tmp/go.pkg -target /
        rm /tmp/go.pkg
    fi
    
    echo -e "${GREEN}✓ Go installed${NC}"
}

# Main installation process
main() {
    # Step 1: Detect OS
    echo -e "${YELLOW}[1/7] Detecting OS and architecture...${NC}"
    detect_os
    echo -e "${GREEN}✓ Detected: $OS_NAME ($OS_ARCH)${NC}"
    echo ""
    
    # Step 2: Check/Install Go
    echo -e "${YELLOW}[2/7] Checking Go installation...${NC}"
    install_go
    echo ""
    
    # Step 3: Install system dependencies
    echo -e "${YELLOW}[3/7] Installing system dependencies...${NC}"
    install_dependencies
    echo -e "${GREEN}✓ Dependencies installed${NC}"
    echo ""
    
    # Step 4: Clone repository if not already in it
    if [ ! -f "go.mod" ] || ! grep -q "cynapse" go.mod; then
        echo -e "${YELLOW}[4/7] Cloning CYNAPSE repository...${NC}"
        TEMP_DIR=$(mktemp -d)
        git clone https://github.com/Alartist40/cynapse.git "$TEMP_DIR/cynapse"
        cd "$TEMP_DIR/cynapse"
        echo -e "${GREEN}✓ Repository cloned${NC}"
    else
        echo -e "${YELLOW}[4/7] Using current directory...${NC}"
        echo -e "${GREEN}✓ Already in CYNAPSE directory${NC}"
    fi
    echo ""
    
    # Step 5: Create ~/.cynapse directory
    echo -e "${YELLOW}[5/7] Setting up CYNAPSE home directory...${NC}"
    mkdir -p ~/.cynapse/synapses
    mkdir -p ~/.cynapse/data/persona
    mkdir -p ~/.cynapse/data/sessions
    mkdir -p ~/.cynapse/logs
    
    # Copy default config if doesn't exist
    if [ ! -f ~/.cynapse/config.yaml ]; then
        if [ -f config.yaml ]; then
            cp config.yaml ~/.cynapse/config.yaml
        fi
    fi
    
    echo -e "${GREEN}✓ Created ~/.cynapse${NC}"
    echo ""
    
    # Step 6: Build CYNAPSE
    echo -e "${YELLOW}[6/7] Building CYNAPSE...${NC}"
    go mod download
    go build -o /tmp/cynapse ./cmd/cynapse
    echo -e "${GREEN}✓ Build complete${NC}"
    echo ""
    
    # Step 7: Install to system PATH
    echo -e "${YELLOW}[7/7] Installing to system PATH...${NC}"
    
    if [ "$OS_TYPE" = "windows" ]; then
        # Windows: install to user's local bin
        mkdir -p ~/bin
        mv /tmp/cynapse ~/bin/cynapse.exe
        echo -e "${YELLOW}Add $HOME/bin to your PATH${NC}"
    else
        # Unix: install to /usr/local/bin
        if [ -w /usr/local/bin ]; then
            mv /tmp/cynapse /usr/local/bin/cynapse
        else
            sudo mv /tmp/cynapse /usr/local/bin/cynapse
        fi
        chmod +x /usr/local/bin/cynapse 2>/dev/null || sudo chmod +x /usr/local/bin/cynapse
    fi
    
    echo -e "${GREEN}✓ Installed to system PATH${NC}"
    echo ""
    
    # Success message
    echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║  ✅ Installation Complete!              ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${BLUE}Run CYNAPSE:${NC}"
    echo -e "  ${YELLOW}cynapse${NC}"
    echo ""
    echo -e "${BLUE}Manage synapses:${NC}"
    echo -e "  ${YELLOW}cynapse synapse list${NC}       - List installed synapses"
    echo -e "  ${YELLOW}cynapse synapse add <name>${NC} - Install a synapse"
    echo ""
    echo -e "${BLUE}Get help:${NC}"
    echo -e "  ${YELLOW}cynapse help${NC}"
    echo ""
}

# Run main installation
main

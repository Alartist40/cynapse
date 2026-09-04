#!/usr/bin/env bash
set -e

# ==============================================================================
# 🧠 CYNAPSE - Single-Line Pure Rust Installer & Hardware Auto-Detector
# ==============================================================================

CYNAPSE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="${HOME}/.local/bin"

echo "======================================================================"
echo "      🧠 CYNAPSE PURE RUST SYSTEM INSTALLER & HARDWARE DETECTOR       "
echo "======================================================================"
echo "Installing from: ${CYNAPSE_DIR}"

# 1. Detect OS Kernel
OS_TYPE="$(uname -s)"
case "${OS_TYPE}" in
    Linux*)     OS="Linux";;
    Darwin*)    OS="macOS";;
    CYGWIN*|MINGW*|MSYS*) OS="Windows";;
    *)          OS="Unknown (${OS_TYPE})";;
esac

# 2. Detect CPU Architecture & Core Count
ARCH="$(uname -m)"
if [ "${OS}" = "macOS" ]; then
    CPU_CORES="$(sysctl -n hw.ncpu 2>/dev/null || echo "4")"
    TOTAL_RAM_BYTES="$(sysctl -n hw.memsize 2>/dev/null || echo "8589934592")"
    TOTAL_RAM_GB=$((TOTAL_RAM_BYTES / 1024 / 1024 / 1024))
else
    CPU_CORES="$(nproc 2>/dev/null || grep -c ^processor /proc/cpuinfo 2>/dev/null || echo "4")"
    TOTAL_RAM_KB="$(grep MemTotal /proc/meminfo 2>/dev/null | awk '{print $2}' || echo "8388608")"
    TOTAL_RAM_GB=$((TOTAL_RAM_KB / 1024 / 1024))
fi

# 3. Detect GPU Acceleration Capabilities
GPU_TYPE="CPU (Software Fallback)"
if command -v nvidia-smi &>/dev/null; then
    NVIDIA_NAME="$(nvidia-smi --query-gpu=gpu_name --format=csv,noheader 2>/dev/null | head -n1 || true)"
    if [ -n "${NVIDIA_NAME}" ]; then
        GPU_TYPE="NVIDIA CUDA (${NVIDIA_NAME})"
    fi
elif [ "${OS}" = "macOS" ] && [ "${ARCH}" = "arm64" ]; then
    GPU_TYPE="Apple Metal Unified Memory"
elif command -v rocm-smi &>/dev/null; then
    GPU_TYPE="AMD ROCm"
fi

echo ""
echo "💻 Hardware Specs Detected:"
echo "   - OS Kernel:   ${OS} (${ARCH})"
echo "   - CPU Cores:   ${CPU_CORES} logical cores"
echo "   - System RAM:  ${TOTAL_RAM_GB} GB RAM"
echo "   - Accelerator: ${GPU_TYPE}"

# 4. Check for Rust Cargo Build System
echo ""
echo "🔍 Toolchain Check:"

if command -v cargo &>/dev/null; then
    RUST_VER="$(cargo --version | head -n1)"
    echo "   [✓] Rust Cargo: ${RUST_VER}"
else
    echo "   [!] Rust Cargo not found. Please install Rust from https://rustup.rs"
    exit 1
fi

# 5. Build Standalone Pure Rust Cynapse Release Binary
echo ""
echo "⚙️ Building Standalone Pure Rust Cynapse Binary (cargo build --release)..."

mkdir -p "${CYNAPSE_DIR}/models" "${CYNAPSE_DIR}/data" "${BIN_DIR}"
(cd "${CYNAPSE_DIR}" && cargo build --release)

# 6. Install Release Executable into User PATH
RELEASE_BIN="${CYNAPSE_DIR}/target/release/cynapse"

if [ -f "${RELEASE_BIN}" ]; then
    cp -f "${RELEASE_BIN}" "${BIN_DIR}/cynapse"
    chmod +x "${BIN_DIR}/cynapse"
    echo "   [✓] Installed release binary to ${BIN_DIR}/cynapse"
else
    echo "   [!] Failed to locate built binary at ${RELEASE_BIN}"
    exit 1
fi

echo ""
echo "======================================================================"
echo "🎉 CYNAPSE INSTALLATION COMPLETE (PURE RUST - ZERO NODE/PYTHON)!"
echo "======================================================================"
echo "Global launcher installed to: ${BIN_DIR}/cynapse"
echo ""
echo "Quick Start Commands:"
echo "  1. Start Interactive Cynapse TUI: cynapse"
echo "  2. List Downloaded Models:         cynapse list"
echo "  3. Test Semantic Router:           cynapse route"
echo ""

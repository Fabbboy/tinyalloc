#!/usr/bin/env sh
set -eu

SRC_DIR="$(pwd)"
BUILD_DIR="$SRC_DIR/.build"

CLEAN=0
for arg in "$@"; do
    case "$arg" in
        --clean)
            CLEAN=1
            ;;
        *)
            echo "[!] Unknown argument: $arg" >&2
            echo "Usage: $0 [--clean]"
            exit 1
            ;;
    esac
done

if [ $CLEAN -eq 1 ]; then
    echo "[*] Cleaning build directory..."
    rm -rf "$BUILD_DIR"
fi

mkdir -p "$BUILD_DIR"
cd "$BUILD_DIR"

echo "[*] Detecting best build system..."
GENERATOR=""
BUILD_CMD=""

if command -v ninja >/dev/null 2>&1; then
    GENERATOR="Ninja"
    BUILD_CMD="ninja"
elif command -v make >/dev/null 2>&1; then
    GENERATOR="Unix Makefiles"
    CORES=$(getconf _NPROCESSORS_ONLN 2>/dev/null || echo 4)
    BUILD_CMD="make -j${CORES}"
else
    echo "[!] No supported build system found (need ninja or make)." >&2
    exit 1
fi

echo "[*] Using generator: $GENERATOR"

cmake -G "$GENERATOR" "$SRC_DIR"
$BUILD_CMD

echo "[✔] Build completed successfully!"

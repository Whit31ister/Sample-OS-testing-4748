#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ISO_PATH="$ROOT_DIR/ISO/sample-os.iso"

if [ ! -f "$ISO_PATH" ]; then
    "$ROOT_DIR/my_os/scripts/build.sh"
fi

qemu-system-x86_64 \
    -cdrom "$ISO_PATH" \
    -display none \
    -monitor none \
    -no-reboot \
    -no-shutdown \
    -serial stdio

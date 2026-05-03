#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
KERNEL_DIR="$ROOT_DIR/my_os/kernel"
STAGING_DIR="$ROOT_DIR/my_os/iso"
OUTPUT_DIR="$ROOT_DIR/ISO"
TARGET_TRIPLE="i686-unknown-linux-gnu"
KERNEL_ELF="$KERNEL_DIR/target/$TARGET_TRIPLE/release/kernel"
ISO_PATH="$OUTPUT_DIR/sample-os.iso"
IMG_PATH="$OUTPUT_DIR/sample-os.img"
RUSTFLAGS_VALUE="-C link-arg=-T$KERNEL_DIR/linker.ld -C link-arg=-nostdlib -C link-arg=-static -C link-arg=-no-pie -C relocation-model=static"

if command -v grub2-mkrescue >/dev/null 2>&1; then
    GRUB_MKRESCUE="grub2-mkrescue"
elif command -v grub-mkrescue >/dev/null 2>&1; then
    GRUB_MKRESCUE="grub-mkrescue"
else
    echo "grub-mkrescue is required to build the bootable image" >&2
    exit 1
fi

if ! command -v xorriso >/dev/null 2>&1; then
    export PATH="$ROOT_DIR/my_os/scripts:$PATH"
fi

mkdir -p "$OUTPUT_DIR" "$STAGING_DIR/boot/grub"

env RUSTFLAGS="$RUSTFLAGS_VALUE" \
    cargo build --manifest-path "$KERNEL_DIR/Cargo.toml" --release --target "$TARGET_TRIPLE"

if command -v grub2-file >/dev/null 2>&1; then
    grub2-file --is-x86-multiboot "$KERNEL_ELF"
fi

cp "$KERNEL_ELF" "$STAGING_DIR/boot/kernel.elf"
cp "$ROOT_DIR/my_os/boot/grub/grub.cfg" "$STAGING_DIR/boot/grub/grub.cfg"

"$GRUB_MKRESCUE" -o "$ISO_PATH" "$STAGING_DIR" >/dev/null
cp "$ISO_PATH" "$IMG_PATH"

if command -v qemu-system-x86_64 >/dev/null 2>&1 && command -v timeout >/dev/null 2>&1; then
    BOOT_LOG="$(mktemp)"

    set +e
    timeout 8 qemu-system-x86_64 \
        -cdrom "$ISO_PATH" \
        -display none \
        -monitor none \
        -no-reboot \
        -no-shutdown \
        -serial "file:$BOOT_LOG" \
        >/dev/null 2>&1
    QEMU_STATUS=$?
    set -e

    if [ "$QEMU_STATUS" -ne 0 ] && [ "$QEMU_STATUS" -ne 124 ]; then
        echo "QEMU boot verification failed" >&2
        rm -f "$BOOT_LOG"
        exit 1
    fi

    if ! grep -q "OS boot successful" "$BOOT_LOG"; then
        echo "Boot verification did not reach the success marker" >&2
        cat "$BOOT_LOG" >&2
        rm -f "$BOOT_LOG"
        exit 1
    fi

    rm -f "$BOOT_LOG"
    printf 'Verified boot in QEMU\n'
fi

printf 'Created %s\n' "$ISO_PATH"
printf 'Created %s\n' "$IMG_PATH"

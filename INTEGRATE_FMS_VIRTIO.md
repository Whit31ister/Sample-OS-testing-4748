# Integrate FMS + VirtIO in `sample-os-ptest`

This folder is now self-contained for integration work.

## What is already integrated

1. Rust modules are embedded inside this folder:
   - `sample-os-ptest/rust/fms-core`
   - `sample-os-ptest/rust/kernel-modules`
   - `sample-os-ptest/rust/os-bridge`
2. Assembly entry now calls Rust:
   - `start` in `src/impl/x86_64/boot/main.asm` calls `rust_kernel_entry`.
3. Build links Rust static library:
   - `Makefile` builds `os-bridge` and links `libos_bridge.a` into `kernel.bin`.

## Build prerequisites

1. `nasm`
2. `x86_64-elf-ld`
3. `grub-mkrescue`
4. Rust toolchain + target:
   - `cargo`
   - `rustup target add x86_64-unknown-none`

## Build command

```sh
cd sample-os-ptest
make build-x86_64
```

## Current integration stage

Right now `rust_kernel_entry` is a minimal bridge to prove successful linking and calling from assembly.

File:
- `sample-os-ptest/rust/os-bridge/src/lib.rs`

## Next step for full VirtIO + FS

To enable real VirtIO block + filesystem:

1. Implement PMM in your kernel and expose Rust-callable hooks:
   - allocate contiguous 4KB pages
   - convert virtual to physical addresses
2. In `os-bridge`, call:
   - `pci_find_virtio_device`
   - `VirtioBlkDevice::init(...)`
   - mount/format with `fms-core`
3. Add a tiny shell/command path in kernel to call FS ops (`files/open/save/delete/info`).

## Copying folder independently

You can copy `sample-os-ptest` anywhere and keep working independently because:

1. Rust crates are local inside `sample-os-ptest/rust`.
2. No dependency paths point outside `sample-os-ptest`.
3. Build logic is inside `sample-os-ptest/Makefile`.

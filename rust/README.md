# Rust Port (FS + PCI + VirtIO)

This folder contains a Rust rewrite of the project in separated modules.

Workspace crates:

1. `fms-core`: `no_std` filesystem core
2. `fms-host`: host CLI for `disk.img` testing
3. `kernel-modules`: `no_std` PCI + VirtIO block + FS glue

## Commands (host CLI)

The Rust CLI keeps your custom commands:

```text
mkfs
files
save
open
delete
info
```

## Build

```sh
cd rust
cargo build
```

## Run Host Smoke Flow

```sh
cargo run -p fms-host -- mkfs disk.img 128
cargo run -p fms-host -- save disk.img ../notes.txt hello.txt
cargo run -p fms-host -- info disk.img hello.txt
cargo run -p fms-host -- open disk.img hello.txt recovered.txt
cargo run -p fms-host -- files disk.img
cargo run -p fms-host -- delete disk.img hello.txt
```

## Kernel Flow

1. Initialize PCI scan (`pci_find_virtio_device`)
2. Initialize VirtIO block (`VirtioBlkDevice::init`)
3. Mount/format filesystem via glue (`fs_mount_over_virtio` / `fs_format_over_virtio`)

You must provide PMM implementation via `PhysicalMemory` trait.

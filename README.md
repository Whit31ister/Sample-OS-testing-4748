# Sample-OS-testing-4748

## Overview

Sample OS Testing 4748 is a minimal, modular operating system project built for learning low-level systems development. It focuses on kernel design, memory management, and hardware interaction using Rust, with GRUB acting as the bootloader. The repository is structured to encourage experimentation and iterative development rather than production stability.

This project serves as a controlled environment for understanding how an OS transitions from boot to execution, and how core subsystems are built from scratch.

---

## Goals

* Understand OS boot flow using GRUB
* Build a kernel in Rust with full ring 0 control
* Explore memory management (paging, allocation)
* Implement interrupts and basic drivers
* Provide a base for custom kernel experimentation

---

## System Architecture

```
+-------------------+
|   BIOS / UEFI     |
+-------------------+
          ↓
+-------------------+
|       GRUB        |
| (Bootloader)      |
+-------------------+
          ↓
+-------------------+
|   Kernel (_start) |
+-------------------+
          ↓
+-------------------+
| Initialization    |
| (GDT, IDT, MMU)   |
+-------------------+
          ↓
+-------------------+
| Core Subsystems   |
| Memory | Drivers  |
| FS     | Tasks    |
+-------------------+
```

---

## Boot Flow

```
Power On
   ↓
Firmware (BIOS/UEFI)
   ↓
GRUB loads kernel.elf into memory
   ↓
GRUB switches CPU mode and passes multiboot info
   ↓
_start() is invoked (Ring 0)
   ↓
Kernel initializes:
   - GDT
   - IDT
   - Memory manager
   - Drivers
   ↓
Kernel enters main loop
```

---

## Project Structure

```
my_os/
│
├── kernel/
│   ├── src/
│   │   ├── main.rs
│   │   ├── arch/
│   │   ├── memory/
│   │   ├── drivers/
│   │   ├── fs/
│   │   └── utils/
│   ├── linker.ld
│   └── Cargo.toml
│
├── boot/grub/
│   └── grub.cfg
│
├── iso/boot/
│   ├── kernel.elf
│   └── grub/grub.cfg
│
├── scripts/
│   ├── build.sh
│   └── run.sh
```

---

## Kernel Initialization Workflow

```
_start()
   ↓
init_all()
   ↓
[ arch::gdt::init() ]
   ↓
[ arch::idt::init() ]
   ↓
[ memory::init() ]
   ↓
[ drivers::init() ]
   ↓
kernel_main_loop()
```

---

## Memory Management Flow

```
Multiboot Info
   ↓
Parse Memory Map
   ↓
Frame Allocator Setup
   ↓
Paging Initialization
   ↓
Heap Allocator Setup
   ↓
Memory Ready for Kernel Use
```

---

## Development Workflow

```
Edit Kernel Code
   ↓
cargo build (custom target)
   ↓
Copy kernel → ISO
   ↓
grub-mkrescue → os.iso
   ↓
Run in QEMU
   ↓
Debug / Iterate
```

---

## Build & Run

```bash
# Build kernel
cargo build --target x86_64-unknown-none

# Create ISO
cp target/.../kernel.elf iso/boot/
grub-mkrescue -o os.iso iso/

# Run
qemu-system-x86_64 -cdrom os.iso
```

---

## Design Principles

* Minimal and transparent implementation
* Strong separation of concerns
* Incremental system growth
* Hardware-first understanding
* Rust for safety with controlled low-level access

---

## Current Status

* GRUB-based boot working
* Kernel entry defined
* Modular structure in place
* Early-stage subsystem development

---

## Future Work

* Paging and virtual memory
* Interrupt handling and timer
* Keyboard driver
* Basic shell
* Filesystem implementation

---

## Disclaimer

This project is experimental and not intended for production use. It prioritizes learning and exploration over completeness, security, or performance.

---

## Summary

Sample OS Testing 4748 is a foundational platform for understanding operating system internals. It provides a structured yet flexible base for building and experimenting with custom kernels while maintaining clarity and control over system behavior.

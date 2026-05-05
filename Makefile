x86_64-asm_source_files := $(shell find src/impl/x86_64 -name '*.asm')
x86_64-asm_objects_files := $(patsubst src/impl/x86_64/%.asm,build/x86_64/%.o,$(x86_64-asm_source_files))
rust_target := x86_64-unknown-none
rust_bridge_lib := rust/target/$(rust_target)/release/libos_bridge.a

$(x86_64-asm_objects_files): build/x86_64/%.o : src/impl/x86_64/%.asm
	mkdir -p $(dir $@) && \
	nasm -f elf64 $< -o $@

.PHONY: build-rust-bridge
build-rust-bridge: check-rust-tools
	cd rust && cargo build -p os-bridge --target $(rust_target) --release

.PHONY: build-x86_64
build-x86_64: check-x86_64-tools build-rust-bridge $(x86_64-asm_objects_files)
	mkdir -p dist/x86_64 && \
	x86_64-elf-ld -n -o dist/x86_64/kernel.bin -T targets/x86_64/linker.ld $(x86_64-asm_objects_files) $(rust_bridge_lib)
	cp dist/x86_64/kernel.bin targets/x86_64/iso/boot/kernel.bin && \
	grub-mkrescue -o dist/x86_64/kernel.iso targets/x86_64/iso

.PHONY: check-x86_64-tools
check-x86_64-tools:
	@command -v nasm >/dev/null 2>&1 || { echo "Missing tool: nasm"; exit 1; }
	@command -v x86_64-elf-ld >/dev/null 2>&1 || { echo "Missing tool: x86_64-elf-ld"; exit 1; }
	@command -v grub-mkrescue >/dev/null 2>&1 || { echo "Missing tool: grub-mkrescue"; exit 1; }

.PHONY: check-rust-tools
check-rust-tools:
	@command -v cargo >/dev/null 2>&1 || { echo "Missing tool: cargo"; exit 1; }
	@rustup target list --installed | grep -q "^$(rust_target)$$" || { echo "Missing rust target: $(rust_target). Run: rustup target add $(rust_target)"; exit 1; }

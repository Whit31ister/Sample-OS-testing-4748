#!/bin/sh
set -eu

DISK="tests/rust-smoke.img"
INPUT="tests/rust-input.txt"
OUTPUT="tests/rust-output.txt"

rm -f "$DISK" "$INPUT" "$OUTPUT"
printf "hello from rust fms\nvirtio-ready layout path\n" > "$INPUT"

cd rust
cargo run -p fms-host -- mkfs "../$DISK" 128
cargo run -p fms-host -- save "../$DISK" "../$INPUT" hello.txt
cargo run -p fms-host -- info "../$DISK" hello.txt
cargo run -p fms-host -- open "../$DISK" hello.txt "../$OUTPUT"
cd ..

cmp "$INPUT" "$OUTPUT"

cd rust
cargo run -p fms-host -- files "../$DISK"
cargo run -p fms-host -- delete "../$DISK" hello.txt
cargo run -p fms-host -- files "../$DISK"
cd ..

rm -f "$DISK" "$INPUT" "$OUTPUT"

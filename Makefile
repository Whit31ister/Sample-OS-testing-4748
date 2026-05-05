.PHONY: rust-build rust-smoke myos-build myos-run clean

rust-build:
	cd rust && cargo build

rust-smoke:
	sh tests/rust-smoke.sh

myos-build:
	sh my_os/scripts/build.sh

myos-run:
	sh my_os/scripts/run.sh

clean:
	rm -rf build
	rm -rf rust/target
	rm -rf my_os/kernel/target
	rm -f ISO/sample-os.iso ISO/sample-os.img
	rm -f rust/disk.img

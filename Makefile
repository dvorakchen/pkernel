env:
	rustup target add riscv64gc-unknown-none-elf
	rustup component add llvm-tools-preview
	cargo install cargo-binutils
	cargo c

build: env
	cargo b --release
	rust-objcopy target/riscv64gc-unknown-none-elf/release/kernel -O binary target/riscv64gc-unknown-none-elf/release/kernel.bin

build_debug: env
	cargo b

debug: build_debug
	qemu-system-riscv64 \
		-machine virt \
		-nographic \
		-bios ../rustsbi-qemu.bin \
		-device loader,file=target/riscv64gc-unknown-none-elf/debug/kernel,addr=0x80200000 \
		-s -S

run: build
	qemu-system-riscv64 \
		-machine virt \
		-nographic \
		-bios ../rustsbi-qemu.bin \
		-device loader,file=target/riscv64gc-unknown-none-elf/release/kernel.bin,addr=0x80200000

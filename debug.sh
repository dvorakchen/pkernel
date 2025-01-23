gdb \
    -ex 'file target/riscv64gc-unknown-none-elf/debug/kernel' \
    -ex 'set arch riscv:rv64' \
    -ex 'target remote localhost:1234'

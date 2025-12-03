default: build

build:
    cargo build --profile=kernel
    mkdir -p target/isofiles/boot/grub
    cp target/x86_64-unknown-none/kernel/wordle_kernel target/isofiles/boot/kernel.bin
    cp grub/grub.cfg target/isofiles/boot/grub/grub.cfg
    i686-elf-grub-mkrescue -o target/wordle.iso target/isofiles

run: build
    qemu-system-x86_64 -cdrom target/wordle.iso
# Wordle Kernel

A work-in-progress bare metal application that runs a wordle clone on x86_64 hardware.

## How to build

Install `just`:

```
cargo install just
```

Edit `justfile`, and change `i686-elf-grub-mkrescue` to your `grub-mkrescue` executable

Run `justfile run`

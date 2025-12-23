default: build

build:
    @mkdir -p build
    @mkdir -p build/wordle
    
    cargo +nightly build --profile=kernel -Z unstable-options --artifact-dir build/wordle

    ./build_limine.sh
    
    @mkdir -p build/iso_root

    @mkdir -p build/iso_root/boot
    cp -v build/wordle/wordle_kernel build/iso_root/boot/
    @mkdir -p build/iso_root/boot/limine
    cp -v limine/limine.conf build/limine/limine-bios.sys build/limine/limine-bios-cd.bin \
      build/limine/limine-uefi-cd.bin build/iso_root/boot/limine/
    
    @mkdir -p build/iso_root/EFI/BOOT
    cp -v build/limine/BOOTX64.EFI build/iso_root/EFI/BOOT/
    cp -v build/limine/BOOTIA32.EFI build/iso_root/EFI/BOOT/

    xorriso -as mkisofs -R -r -J -b boot/limine/limine-bios-cd.bin \
        -no-emul-boot -boot-load-size 4 -boot-info-table -hfsplus \
        -apm-block-size 2048 --efi-boot boot/limine/limine-uefi-cd.bin \
        -efi-boot-part --efi-boot-image --protective-msdos-label \
        build/iso_root -o build/image.iso

    ./build/limine/limine bios-install build/image.iso

run-bios: build
    qemu-system-x86_64 -cdrom build/image.iso

run-uefi: build
    qemu-system-x86_64 --bios bios.bin -cdrom build/image.iso -net none -d cpu_reset

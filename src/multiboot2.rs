use core::u32;

const MULTIBOOT2_MAGIC: u32 = 0xE85250D6;

#[repr(C)]
struct Multiboot2Header {
    magic: u32,
    architecture: u32,
    header_length: u32,
    checksum: u32,
    tags: Tag<()>,
}

#[repr(C)]
struct Tag<T: Sized> {
    r#type: u16,
    flags: u16,
    size: u32,
    payload: T,
}

#[used]
#[unsafe(link_section = "multiboot_header")]
static MULTIBOOT2_HEADER: Multiboot2Header = Multiboot2Header {
    magic: MULTIBOOT2_MAGIC,
    architecture: 0, // i386
    header_length: size_of::<u32>() as u32 * 4 + size_of::<Tag<()>>() as u32,
    checksum: u32::MAX // subtract from max to avoid underflow
        - MULTIBOOT2_MAGIC
        - 0
        - size_of::<u32>() as u32 * 4
        - size_of::<Tag<()>>() as u32
        + 1, // add one to compensate for subtracting from max instead of 0.
    tags: Tag {
        // end tag
        r#type: 0,
        flags: 0,
        size: 8,
        payload: (),
    },
};

use limine::memory_map::EntryType;
use x86_64::{
    PhysAddr, VirtAddr,
    registers::control::{Cr3, Cr3Flags},
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageSize, PageTable, PageTableFlags,
        PhysFrame, Size1GiB, Size2MiB, Size4KiB,
    },
};

use crate::limine_requests::{EXECUTABLE_ADDRESS_REQUEST, HHDM_REQUEST, MEMMAP_REQUEST};

// Linker symbols placed at the boundaries of kernel sections
unsafe extern "C" {
    static __text_start: u8;
    static __text_end: u8;
    static __rodata_start: u8;
    static __rodata_end: u8;
    static __data_start: u8;
    static __data_end: u8;
}

pub const HHDM_OFFSET: u64 = 0xffff_8000_0000_0000;

// These two must be 0x1000 aligned.
pub const STACK_BASE: u64 = 0x4888_8888_0000;
pub const STACK_SIZE: u64 = 0x1000 * 16;

pub fn initialize_paging() {
    // TODO: Setup HHDM / Stack / Heap
    let hhdm_offset = HHDM_REQUEST
        .get_response()
        .expect("Response should be provided by Limine.")
        .offset();

    // Allocate a new frame for the new page table
    let mut frame_allocator = EarlyFrameAllocator::new();
    let page_table_addr = frame_allocator
        .allocate_frame()
        .expect("Out of memory.")
        .start_address()
        .as_u64()
        + hhdm_offset;

    let page_table_ptr = page_table_addr as *mut PageTable;

    // SAFETY: Frame allocator should return a valid page of memory to put the top level page table
    // into.
    unsafe {
        *page_table_ptr = PageTable::new();
    }

    // This implementation of mapper uses an offset value to calculate physical addresses from
    // virtual addresses.
    // SAFETY: offset is provided by limine and should be correct; pointer should point to valid
    // page table as we initialized it before.
    let mut offset_page_table =
        unsafe { OffsetPageTable::new(&mut *page_table_ptr, VirtAddr::new(hhdm_offset)) };

    map_hhdm(&mut offset_page_table, &mut frame_allocator);
    map_kernel(&mut offset_page_table, &mut frame_allocator);
    map_stack(&mut offset_page_table, &mut frame_allocator);

    // SAFETY: after switching to our own paging we will also be switching stack and calling a new
    // entry point function. This means we won't rely on any references that still used the old
    // memory layout.
    // Any references to kernel stuff also should be valid as we map the kernel to the same place.
    unsafe {
        Cr3::write(
            PhysFrame::from_start_address_unchecked(PhysAddr::new(page_table_addr - hhdm_offset)),
            Cr3Flags::empty(),
        );
    }
}

fn map_hhdm(offset_page_table: &mut OffsetPageTable, frame_allocator: &mut EarlyFrameAllocator) {
    for entry in MEMMAP_REQUEST
        .get_response()
        .expect("Response should be provided by Limine.")
        .entries()
    {
        // Don't map unusable memory
        if entry.entry_type == EntryType::RESERVED || entry.entry_type == EntryType::BAD_MEMORY {
            continue;
        }

        let flags = PageTableFlags::PRESENT | PageTableFlags::NO_EXECUTE | PageTableFlags::WRITABLE;

        map_range(
            offset_page_table,
            frame_allocator,
            PhysAddr::new(entry.base),
            VirtAddr::new(entry.base + HHDM_OFFSET),
            entry.length,
            flags,
        );
    }
}

fn map_kernel(offset_page_table: &mut OffsetPageTable, frame_allocator: &mut EarlyFrameAllocator) {
    let mut phys = EXECUTABLE_ADDRESS_REQUEST
        .get_response()
        .expect("Response should be provided by Limine.")
        .physical_base();

    let text_length = (&raw const __text_end) as u64 - (&raw const __text_start) as u64;
    map_range(
        offset_page_table,
        frame_allocator,
        PhysAddr::new(phys),
        VirtAddr::from_ptr(&raw const __text_start),
        text_length,
        PageTableFlags::PRESENT,
    );
    // Kernel executable is guaranteed to be physically contiguous according to Limine.
    phys += text_length;

    let rodata_length = (&raw const __rodata_end) as u64 - (&raw const __rodata_start) as u64;
    map_range(
        offset_page_table,
        frame_allocator,
        PhysAddr::new(phys),
        VirtAddr::from_ptr(&raw const __rodata_start),
        rodata_length,
        PageTableFlags::PRESENT | PageTableFlags::NO_EXECUTE,
    );
    phys += rodata_length;

    let data_length = (&raw const __data_end) as u64 - (&raw const __data_start) as u64;
    map_range(
        offset_page_table,
        frame_allocator,
        PhysAddr::new(phys),
        VirtAddr::from_ptr(&raw const __data_start),
        data_length,
        PageTableFlags::PRESENT | PageTableFlags::NO_EXECUTE | PageTableFlags::WRITABLE,
    );
}

fn map_stack(offset_page_table: &mut OffsetPageTable, frame_allocator: &mut EarlyFrameAllocator) {
    // i represents number of pages
    for i in 0u64..STACK_SIZE / 0x1000 {
        let frame = frame_allocator.allocate_frame().expect("Out of memory.");

        // SAFETY: Stack is only mapped once.
        unsafe {
            offset_page_table
                .map_to(
                    Page::from_start_address_unchecked(VirtAddr::new(STACK_BASE + i * 0x1000)),
                    frame,
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE,
                    frame_allocator,
                )
                .expect("Mapping failed.")
                .ignore();
        }
    }
}

fn map_range(
    offset_page_table: &mut OffsetPageTable,
    frame_allocator: &mut EarlyFrameAllocator,
    mut phys: PhysAddr,
    mut virt: VirtAddr,
    size: u64,
    flags: PageTableFlags,
) {
    let end = virt + size;

    while virt < end {
        // 1GiB pages
        if virt.is_aligned(Size1GiB::SIZE)
            && phys.is_aligned(Size1GiB::SIZE)
            && virt + Size1GiB::SIZE <= end
        {
            let page = Page::<Size1GiB>::from_start_address(virt).unwrap();
            let frame = PhysFrame::<Size1GiB>::from_start_address(phys).unwrap();

            unsafe {
                offset_page_table
                    .map_to(page, frame, flags, frame_allocator)
                    .expect("Mapping failed.")
                    .ignore();
                // We ignore the result because we are building a new page table
                // which isn't active yet. We will switch the page tables all at
                // once later.
            }

            virt += Size1GiB::SIZE;
            phys += Size1GiB::SIZE;
            continue;
        }

        // 2MiB pages
        if virt.is_aligned(Size2MiB::SIZE)
            && phys.is_aligned(Size2MiB::SIZE)
            && virt + Size2MiB::SIZE <= end
        {
            let page = Page::<Size2MiB>::from_start_address(virt).unwrap();
            let frame = PhysFrame::<Size2MiB>::from_start_address(phys).unwrap();

            unsafe {
                offset_page_table
                    .map_to(page, frame, flags, frame_allocator)
                    .expect("Mapping failed.")
                    .ignore();
            }

            virt += Size2MiB::SIZE;
            phys += Size2MiB::SIZE;
            continue;
        }

        // 4KiB pages
        let page = Page::<Size4KiB>::from_start_address(virt).unwrap();
        let frame = PhysFrame::<Size4KiB>::from_start_address(phys).unwrap();

        unsafe {
            offset_page_table
                .map_to(page, frame, flags, frame_allocator)
                .expect("Mapping failed.")
                .ignore();
        }

        virt += Size4KiB::SIZE;
        phys += Size4KiB::SIZE;
    }
}

// This allocator completely ignores reclaimable memory. It only allocates from USABLE
struct EarlyFrameAllocator {
    memmap_index: usize,
    next_frame: PhysAddr,
}

impl EarlyFrameAllocator {
    pub fn new() -> Self {
        Self {
            memmap_index: 0,
            next_frame: PhysAddr::zero(),
        }
    }
}

// SAFETY: next_frame pointer is always updated after a frame has been allocated so it will not
// allocate any frames that has already been allocated.
unsafe impl FrameAllocator<Size4KiB> for EarlyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<x86_64::structures::paging::PhysFrame<Size4KiB>> {
        for entry in MEMMAP_REQUEST
            .get_response()
            .expect("Response should be provided by Limine.")
            .entries()[self.memmap_index..]
            .iter()
        {
            // If not usable, move on to the next region
            if entry.entry_type != EntryType::USABLE {
                self.memmap_index += 1;
                continue;
            }

            // If the next_frame pointer is after the current region, move on to the next region.
            if self.next_frame.as_u64() >= entry.base + entry.length {
                self.memmap_index += 1;
                continue;
            }

            // Before the current region in the memory map.
            if self.next_frame.as_u64() < entry.base {
                self.next_frame = PhysAddr::new(entry.base);
            }

            // Allocate one page
            self.next_frame += 0x1000;
            // SAFETY: USABLE entry bases are guaranteed to be page aligned; adding 0x1000 to any
            // page aligned address yields a page aligned result; 0x0 is page aligned.
            return Some(unsafe {
                PhysFrame::from_start_address_unchecked(self.next_frame - 0x1000)
            });
        }

        None
    }
}

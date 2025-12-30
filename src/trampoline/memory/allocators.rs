// SPDX-FileCopyrightText: 2025 SunnyMonster
//
// SPDX-License-Identifier: GPL-3.0-or-later

use alloc::vec;
use alloc::vec::Vec;
use limine::memory_map::EntryType;
use x86_64::{
    PhysAddr,
    structures::paging::{FrameAllocator, PageSize, PhysFrame, Size4KiB},
};

use crate::trampoline::{
    limine_requests::MEMMAP_REQUEST,
    memory::{ALLOCATOR, HEAP_BASE, HEAP_SIZE},
};

/// # Safety
/// This function should only be called after the heap has been mapped.
pub unsafe fn init_allocator() {
    unsafe {
        ALLOCATOR
            .lock()
            .init(HEAP_BASE as *mut u8, HEAP_SIZE as usize);
    }
}

// This allocator completely ignores reclaimable memory. It only allocates from USABLE
pub struct EarlyFrameAllocator {
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

impl Default for EarlyFrameAllocator {
    fn default() -> Self {
        Self::new()
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

pub struct AddressRange(u64, u64);

pub struct ProperFrameAllocator {
    availables: Vec<AddressRange>,
}

impl ProperFrameAllocator {
    fn push_range(availables: &mut Vec<AddressRange>, range: AddressRange) {
        // If the last range ends the same as the new range's start, we can merge them
        if let Some(last) = availables.last_mut()
            && last.1 == range.0
        {
            last.1 = range.1;
            return;
        }

        // Otherwise we just push it
        availables.push(range);
    }

    pub fn new(early_frame_allocator: EarlyFrameAllocator) -> Self {
        let mut availables = vec![];

        for entry in MEMMAP_REQUEST
            .get_response()
            .expect("Response should be provided by Limine.")
            .entries()
        {
            match entry.entry_type {
                // Early frame allocator does not allocate from reclaimable memory
                EntryType::BOOTLOADER_RECLAIMABLE | EntryType::ACPI_RECLAIMABLE => {
                    Self::push_range(
                        &mut availables,
                        AddressRange(entry.base, entry.base + entry.length),
                    );
                }
                // Only usable sections that haven't been touched by the early frame allocator can
                // be used.
                EntryType::USABLE => {
                    // The next frame pointer has already moved past this USABLE section
                    if entry.base + entry.length <= early_frame_allocator.next_frame.as_u64() {
                        continue;
                    }

                    // The next frame pointer is inside this USABLE section
                    if early_frame_allocator.next_frame.as_u64() >= entry.base {
                        Self::push_range(
                            &mut availables,
                            AddressRange(
                                early_frame_allocator.next_frame.as_u64(),
                                entry.base + entry.length,
                            ),
                        );
                        continue;
                    }

                    // The next frame pointer is before this USABLE section
                    Self::push_range(
                        &mut availables,
                        AddressRange(entry.base, entry.base + entry.length),
                    );
                }
                _ => {}
            }
        }

        Self { availables }
    }
}

unsafe impl<S: PageSize> FrameAllocator<S> for ProperFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<S>> {
        let mut additional_range = None;
        let mut frame_start = None;

        'outer: for range in &mut self.availables {
            let mut start = range.0;

            while range.1 - start >= S::SIZE {
                let align_mask = !(S::SIZE - 1);

                // The address is aligned
                if start & align_mask == start {
                    let range_end = range.1;
                    range.1 = start;

                    if start + S::SIZE < range_end {
                        additional_range = Some(AddressRange(start + S::SIZE, range_end));
                    }

                    frame_start = Some(start);

                    break 'outer;
                }

                start += Size4KiB::SIZE;
            }
        }

        // This will put the new range at the back, which means it can't merge with other ranges if
        // any ranges are then freed. However, in this project, we don't need to free frames ever.
        // So that's fine.
        if let Some(additional_range) = additional_range {
            self.availables.push(additional_range);
        }

        frame_start.map(|a| {
            PhysFrame::from_start_address(PhysAddr::new(a))
                .expect("Address should always be correctly aligned.")
        })
    }
}

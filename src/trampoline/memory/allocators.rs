// SPDX-FileCopyrightText: 2025 SunnyMonster
//
// SPDX-License-Identifier: GPL-3.0-or-later

use alloc::vec;
use alloc::vec::Vec;
use limine::memory_map::EntryType;
use x86_64::{
    PhysAddr,
    structures::paging::{FrameAllocator, PageSize, PhysFrame, Size4KiB, frame::PhysFrameRange},
};

use crate::trampoline::{
    limine_requests::MEMMAP_REQUEST,
    memory::{ALLOCATOR, HEAP_BASE, HEAP_PAGES},
};

/// # Safety
/// This function should only be called after the heap has been mapped.
pub unsafe fn init_allocator() {
    unsafe {
        ALLOCATOR.lock().init(
            HEAP_BASE.start_address().as_u64() as *mut u8,
            HEAP_PAGES as usize * Size4KiB::SIZE as usize,
        );
    }
}

// This allocator completely ignores reclaimable memory. It only allocates from USABLE
pub struct EarlyFrameAllocator {
    memmap_index: usize,
    next_frame: PhysFrame,
}

impl EarlyFrameAllocator {
    pub fn new() -> Self {
        Self {
            memmap_index: 0,
            // SAFETY: Zero address should be correctly aligned.
            next_frame: unsafe { PhysFrame::from_start_address_unchecked(PhysAddr::zero()) },
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
            if self.next_frame.start_address().as_u64() >= entry.base + entry.length {
                self.memmap_index += 1;
                continue;
            }

            // Before the current region in the memory map.
            if self.next_frame.start_address().as_u64() < entry.base {
                self.next_frame =
                    // SAFETY: Address returned by limine memmap should be properly aligned.
                    unsafe { PhysFrame::from_start_address_unchecked(PhysAddr::new(entry.base)) };
            }

            // Allocate one page
            self.next_frame += 1;
            return Some(self.next_frame - 1);
        }

        None
    }
}

pub struct ProperFrameAllocator {
    availables: Vec<PhysFrameRange>,
}

/// # SAFETY
/// The caller must ensure the base and the size are properly aligned.
unsafe fn address_range_unchecked<S: PageSize>(base: u64, size: u64) -> PhysFrameRange<S> {
    // SAFETY: Caller ensures address and size are properly aligned.
    unsafe {
        PhysFrameRange {
            start: PhysFrame::from_start_address_unchecked(PhysAddr::new(base)),
            end: PhysFrame::from_start_address_unchecked(PhysAddr::new(base + size)),
        }
    }
}

impl ProperFrameAllocator {
    fn push_range(availables: &mut Vec<PhysFrameRange>, range: PhysFrameRange) {
        // If the last range ends the same as the new range's start, we can merge them
        if let Some(last) = availables.last_mut()
            && last.end == range.start
        {
            last.end = range.end;
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
                    // SAFETY: memmap entries by Limine should be aligned.
                    Self::push_range(&mut availables, unsafe {
                        address_range_unchecked(entry.base, entry.length)
                    });
                }
                // Only usable sections that haven't been touched by the early frame allocator can
                // be used.
                EntryType::USABLE => {
                    // The next frame pointer has already moved past this USABLE section
                    if entry.base + entry.length
                        <= early_frame_allocator.next_frame.start_address().as_u64()
                    {
                        continue;
                    }

                    // The next frame pointer is inside this USABLE section
                    if early_frame_allocator.next_frame.start_address().as_u64() >= entry.base {
                        // SAFETY: Frame address must be aligned, length from Limine should be
                        // aligned.
                        Self::push_range(&mut availables, unsafe {
                            address_range_unchecked(
                                early_frame_allocator.next_frame.start_address().as_u64(),
                                entry.length,
                            )
                        });
                        continue;
                    }

                    // The next frame pointer is before this USABLE section
                    // SAFETY: Addresses from Limine memmap should be aligned.
                    Self::push_range(&mut availables, unsafe {
                        address_range_unchecked(entry.base, entry.length)
                    });
                }
                _ => {}
            }
        }

        Self { availables }
    }
}

// SAFETY: This allocator builds a list of available regions based on the state of the
// EarlyFrameAllocator which existed prior. It only allocates from available regions.
unsafe impl<S: PageSize> FrameAllocator<S> for ProperFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<S>> {
        let mut additional_range = None;
        let mut frame_start = None;

        'outer: for range in &mut self.availables {
            let mut start = range.start;

            while range.end - start >= S::SIZE / Size4KiB::SIZE {
                if start.start_address().is_aligned(S::SIZE) {
                    let range_end = range.end;
                    range.end = start;

                    // If allocating 1 frame results in left-over space
                    if start + S::SIZE / Size4KiB::SIZE < range_end {
                        additional_range = Some(PhysFrameRange {
                            start: start + S::SIZE / Size4KiB::SIZE,
                            end: range_end,
                        });
                    }

                    frame_start = Some(start);

                    break 'outer;
                }

                start += 1;
            }
        }

        // This will put the new range at the back, which means it can't merge with other ranges if
        // any ranges are then freed. However, in this project, we don't need to free frames ever.
        // So that's fine.
        if let Some(additional_range) = additional_range {
            self.availables.push(additional_range);
        }

        frame_start.map(|a| {
            PhysFrame::from_start_address(a.start_address())
                .expect("Address should always be correctly aligned.")
        })
    }
}

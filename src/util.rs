// SPDX-FileCopyrightText: 2025 SunnyMonster
//
// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::Infallible;

use x86_64::{
    VirtAddr,
    structures::paging::{Page, PageSize},
};

pub trait InfallibleResultExt<T> {
    fn infallible(self) -> T;
}

impl<T> InfallibleResultExt<T> for Result<T, Infallible> {
    fn infallible(self) -> T {
        self.expect("Result is infallible.")
    }
}

pub const fn page_from_addr<S: PageSize>(addr: u64) -> Page<S> {
    assert!(addr.is_multiple_of(S::SIZE));
    // SAFETY: Alignment asserted in previous line.
    unsafe { Page::from_start_address_unchecked(VirtAddr::new(addr)) }
}

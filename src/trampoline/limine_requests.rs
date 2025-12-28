// SPDX-FileCopyrightText: 2025 SunnyMonster
//
// SPDX-License-Identifier: GPL-3.0-or-later

use limine::{
    BaseRevision,
    request::{
        ExecutableAddressRequest, FramebufferRequest, HhdmRequest, MemoryMapRequest,
        RequestsEndMarker, RequestsStartMarker,
    },
};

// SAFETY (for all): the link sections are defined in the linker script to reside near the top of
// the executable and in the right order, so that the bootloader can read them.
#[used]
#[unsafe(link_section = ".requests_start_marker")]
pub static _REQUESTS_START_MARKER: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[unsafe(link_section = ".requests")]
pub static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests")]
pub static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
pub static MEMMAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
pub static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
pub static EXECUTABLE_ADDRESS_REQUEST: ExecutableAddressRequest = ExecutableAddressRequest::new();

#[used]
#[unsafe(link_section = ".requests_end_marker")]
pub static _REQUESTS_END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

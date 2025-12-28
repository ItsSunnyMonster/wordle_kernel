// SPDX-FileCopyrightText: 2025 SunnyMonster
//
// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::Infallible;

pub trait InfallibleResultExt<T> {
    fn infallible(self) -> T;
}

impl<T> InfallibleResultExt<T> for Result<T, Infallible> {
    fn infallible(self) -> T {
        self.expect("Result is infallible.")
    }
}

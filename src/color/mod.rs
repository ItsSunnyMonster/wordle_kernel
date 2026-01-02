// SPDX-FileCopyrightText: 2026 SunnyMonster
//
// SPDX-License-Identifier: GPL-3.0-or-later

use embedded_graphics::pixelcolor::Rgb888;

macro_rules! color_impl {
    ($func_name:ident, $color:ident) => {
        fn $func_name(&self) -> Rgb888 {
            let color = self.colors.$color;
            Rgb888::new(color.rgb.r, color.rgb.g, color.rgb.b)
        }
    };
}

pub const COLOR_SCHEME: catppuccin::Flavor = catppuccin::PALETTE.mocha;

pub trait ColorScheme {
    fn background(&self) -> Rgb888;
    fn error_foreground(&self) -> Rgb888;
}

impl ColorScheme for catppuccin::Flavor {
    color_impl!(background, crust);
    color_impl!(error_foreground, red);
}

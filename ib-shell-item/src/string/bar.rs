use std::cmp;

use bon::Builder;
use widestring::{Utf16String, utf16str};

/// Block characters from empty to full (8 steps).
const BLOCKS: [char; 8] = [
    '\u{258F}', '\u{258E}', '\u{258D}', '\u{258C}', '\u{258B}', '\u{258A}', '\u{2589}', '\u{2588}',
];
const BLOCKS_N: u64 = BLOCKS.len() as u64;

/*
#[derive(Debug, Clone, Copy, Default)]
pub enum HorizontalAlignment {
    #[default]
    Left,
    Center,
    Right,
}
*/

/**
Make plain text bars with Unicode
[block elements](https://en.wikipedia.org/wiki/Block_Elements).

## Monospaced vs. proportional fonts
It is easier to make string bars with proportional fonts,
which often looks better too.
But these string bars look bad with monospaced fonts.
On the other side, monospaced bars look okay with proportional fonts.

Windows 11 File Explorer uses a proportional font by default.
However, Windows 10 uses a monospaced one;
and even on Windows 11 the user may use tools like MacType to
customize the font.

So we use monospaced mode by default, but if you know the app will
use a proportional font, you should use proportional mode;
or provide an option to the user.

Related issues:
- [资源管理器文件夹大小集成功能在win10的显示问题 - Issue #112 - IbEverythingExt](https://github.com/Chaoses-Ib/IbEverythingExt/issues/112)

## Alignment
Unfortunately, Unicode only provides full block elements for lower and left variants.
For right variants, there are only 4/8 and 1/8 blocks, and some fonts don't even support them,
like Microsoft YaHei.

So we only provide left alignment to simplify the implementation.
*/
#[derive(Builder, Debug, Clone)]
pub struct StringBar {
    value: u64,

    /// [`StringBar::value`] is allowed to be larger than [`StringBar::max`].
    max: u64,

    /// In 1/8-block units.
    ///
    /// - File Explorer:
    ///   This is equivalent to device-independent pixels (in default scale).
    ///   If too wide, the column will be truncated from right to left, even it's right-aligned.
    width: u16,

    /// For bars too short, return a minimum bar instead an empty string.
    #[builder(default)]
    min_bar: bool,

    /*
    #[builder(default)]
    alignment: HorizontalAlignment,
    */
    /// See [`StringBar`] for details.
    #[builder(default)]
    proportional_font: bool,
}

impl StringBar {
    const fn width(&self) -> u64 {
        self.width as _
    }

    fn min_bar(&self) -> Utf16String {
        if self.min_bar {
            utf16str!("\u{258F}").into()
        } else {
            Default::default()
        }
    }

    pub fn to_utf16_string(&self) -> Utf16String {
        if self.proportional_font {
            self.proportional_font_to_utf16_string()
        } else {
            self.monospaced_font_to_utf16_string()
        }
    }

    /// For proportional fonts, making bar is easy.
    ///
    /// Unfortunately, we also can't overlap bar with label string easily.
    ///
    /// See [`StringBar`] for details.
    pub fn proportional_font_to_utf16_string(&self) -> Utf16String {
        let i = self.min_bar as u64;
        let n = if self.max == 0 {
            i
        } else {
            cmp::min(
                (self.value.saturating_mul(self.width()) / self.max) + i,
                self.width(),
            )
        };
        let bar = utf16str!("\u{258F}").repeat(n as usize);
        bar
    }

    /// See [`StringBar`] for details.
    pub fn monospaced_font_to_utf16_string(&self) -> Utf16String {
        if self.max == 0 {
            return self.min_bar();
        }

        // The total number of 1/8-block units that should be filled across the entire bar
        let units = self.value.saturating_mul(self.width()) / self.max;
        if units == 0 {
            return self.min_bar();
        }

        let full_blocks = units / BLOCKS_N;
        let rem = units % BLOCKS_N;

        let capacity = (full_blocks + ((rem > 0) as u64)) as usize;
        let mut bar = Utf16String::with_capacity(capacity);

        /*
        match self.alignment {
            HorizontalAlignment::Right => {
                if rem > 0 {
                    result.push(BLOCKS[rem as usize - 1]);
                }
            }
            _ => {}
        }
        */
        for _ in 0..full_blocks {
            bar.push(*BLOCKS.last().unwrap());
        }
        /*
        match self.alignment {
            HorizontalAlignment::Left | HorizontalAlignment::Center => {
                if rem > 0 {
                    result.push(BLOCKS[rem as usize - 1]);
                }
            }
            _ => {}
        }
        */
        if rem > 0 {
            bar.push(BLOCKS[rem as usize - 1]);
        }

        bar
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let bar = StringBar::builder().value(0).max(100).width(80).build();
        let result = bar.to_utf16_string();
        assert!(result.is_empty());

        let bar = StringBar::builder()
            .value(0)
            .max(100)
            .width(80)
            .min_bar(true)
            .build();
        let result = bar.to_utf16_string();
        assert_eq!(result.to_string(), "\u{258F}");
    }

    #[test]
    fn zero_max() {
        let bar = StringBar::builder().value(5).max(0).width(80).build();
        let result = bar.to_utf16_string();
        assert!(result.is_empty());

        let bar = StringBar::builder()
            .value(5)
            .max(0)
            .width(80)
            .min_bar(true)
            .build();
        let result = bar.to_utf16_string();
        assert_eq!(result.to_string(), "\u{258F}");
    }

    #[test]
    fn half_full() {
        let bar = StringBar::builder().value(50).max(100).width(80).build();
        let result = bar.to_utf16_string();
        assert_eq!(result.len(), 5);
        assert_eq!(result.to_string(), "█████");
    }

    #[test]
    fn full() {
        let bar = StringBar::builder().value(100).max(100).width(80).build();
        let result = bar.to_utf16_string();
        assert_eq!(result.len(), 10);
        assert_eq!(result.to_string(), "██████████");
    }

    #[test]
    fn partial_block_left() {
        let bar = StringBar::builder().value(53).max(100).width(80).build();
        let result = bar.to_utf16_string();
        // 53% of 80 units = 42.4 units → 5 full blocks (40 units) + 2 units partial
        assert_eq!(result.len(), 6);
        assert_eq!(result.to_string(), "█████▎");
        assert_eq!(result.chars().nth(5).unwrap(), '\u{258E}');
    }

    /*
    #[test]
    fn partial_block_right() {
        let bar = StringBar::builder()
            .current(53)
            .max(100)
            .width(80)
            .alignment(HorizontalAlignment::Right)
            .build();
        let result = bar.to_utf16_string();
        // Right: partial first → "▎█████"
        assert_eq!(result.len(), 6);
        assert_eq!(result.chars().nth(0).unwrap(), '\u{258E}');
    }
    */
}

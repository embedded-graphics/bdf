//! eg-bdf: BDF font support for embedded-graphics.

#![no_std]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(missing_copy_implementations)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![deny(unsafe_code)]
#![deny(unstable_features)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]

use embedded_graphics::{
    iterator::raw::RawDataSlice,
    pixelcolor::raw::{LittleEndian, RawU1},
    prelude::*,
    primitives::Rectangle,
};

mod text;
pub use text::BdfTextStyle;

/// BDF font.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BdfFont<'a> {
    /// The index of the replacement character.
    pub replacement_character: usize,
    /// The ascent in pixels.
    pub ascent: u32,
    /// The descent in pixels.
    pub descent: u32,
    /// The glyph information.
    pub glyphs: &'a [BdfGlyph],
    /// The bitmap data.
    pub data: &'a [u8],
}

impl<'a> BdfFont<'a> {
    fn get_glyph(&self, c: char) -> &'a BdfGlyph {
        self.glyphs
            .iter()
            .find(|g| g.character == c)
            // TODO: don't panic if replacement_character is invalid
            .unwrap_or_else(|| &self.glyphs[self.replacement_character])
    }
}

/// BDF glyph information.
// TODO: store more efficiently (e.g. use smaller integer types if possible, store as struct of arrays instead of array of structs)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BdfGlyph {
    /// The corresponding character.
    pub character: char,
    /// The glyph bounding box.
    pub bounding_box: Rectangle,
    /// The horizontal distance to the start point of the next glyph.
    pub device_width: u32,
    /// The start index in the bitmap data.
    pub start_index: usize,
}

impl BdfGlyph {
    fn draw<D: DrawTarget>(
        &self,
        position: Point,
        color: D::Color,
        data: &[u8],
        target: &mut D,
    ) -> Result<(), D::Error> {
        let mut data_iter = RawDataSlice::<RawU1, LittleEndian>::new(data).into_iter();

        if self.start_index > 0 {
            data_iter.nth(self.start_index - 1);
        }

        self.bounding_box
            .translate(position)
            .points()
            .filter_map(|p| {
                if data_iter.next()? == RawU1::new(1) {
                    Some(Pixel(p, color))
                } else {
                    None
                }
            })
            .draw(target)
    }
}

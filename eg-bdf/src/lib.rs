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

mod proportional;
mod serialized;
pub use proportional::{ProportionalFont, ProportionalTextStyle};
pub use serialized::{SerializedBdfFont, SerializedBdfTextStyle};

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

/// BDF glyph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BdfGlyph {
    /// The corresponding character.
    pub character: char,
    /// The glyph bounding box.
    pub bounding_box: Rectangle,
    /// The horizontal distance to the start point of the next glyph.
    pub device_width: u32,
    /// The bitmap data of the glyph.
    pub start_index: usize,
}

impl<'a> BdfGlyph {
    fn into_glyph(self, font: &'a BdfFont) -> DisplayBdfGlyph<'a> {
        DisplayBdfGlyph {
            character: self.character,
            bounding_box: self.bounding_box,
            device_width: self.device_width,
            bitmap_data: &font.data[self.start_index..],
        }
    }
}

/// Unserialized BDF text style
pub type BdfTextStyle<'a, C> = ProportionalTextStyle<'a, BdfFont<'a>, C>;

/// BDF glyph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DisplayBdfGlyph<'a> {
    /// The corresponding character.
    pub character: char,
    /// The glyph bounding box.
    pub bounding_box: Rectangle,
    /// The horizontal distance to the start point of the next glyph.
    pub device_width: u32,
    /// The bitmap data of the glyph.
    pub bitmap_data: &'a [u8],
}

impl<'a> DisplayBdfGlyph<'a> {
    /// Draws a glyph at a certain place and color
    pub fn draw<D: DrawTarget>(
        &self,
        position: Point,
        color: D::Color,
        target: &mut D,
    ) -> Result<(), D::Error> {
        let mut data_iter = RawDataSlice::<RawU1, LittleEndian>::new(self.bitmap_data).into_iter();

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

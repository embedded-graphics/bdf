#![no_std]

use embedded_graphics::{
    iterator::raw::RawDataSlice,
    pixelcolor::raw::{LittleEndian, RawU1},
    prelude::*,
    primitives::Rectangle,
};

pub use eg_bdf_macros::include_bdf;

pub mod text;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BdfFont<'a> {
    pub replacement_character: usize,
    pub line_height: u32,
    pub glyphs: &'a [BdfGlyph],
    pub data: &'a [u8],
}

impl<'a> BdfFont<'a> {
    fn get_glyph(&self, c: char) -> &'a BdfGlyph {
        self.glyphs
            .iter()
            .find(|g| g.character == c)
            .unwrap_or_else(|| &self.glyphs[self.replacement_character])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BdfGlyph {
    pub character: char,
    pub bounding_box: Rectangle,
    pub device_width: u32,
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
            .zip(data_iter)
            .filter(|(_p, c)| *c == RawU1::new(1))
            .map(|(p, _c)| Pixel(p, color))
            .draw(target)
    }
}

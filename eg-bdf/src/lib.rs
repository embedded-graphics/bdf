#![no_std]

use embedded_graphics::{prelude::*, primitives::Rectangle};

pub use eg_bdf_macros::include_bdf;

pub mod text;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BdfFont<'a, 'b> {
    pub glyphs: &'a [BdfGlyph<'b>],
}

impl BdfFont<'_, '_> {
    fn get_glyph(&self, c: char) -> Option<&BdfGlyph> {
        self.glyphs.iter().find(|g| g.character == c)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BdfGlyph<'a> {
    pub character: char,
    pub bounding_box: Rectangle,
    pub device_width: u32,
    pub data: &'a [u8],
}

impl BdfGlyph<'_> {
    fn draw<D>(&self, mut position: Point, color: D::Color, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget,
    {
        let bytes_per_row = (self.bounding_box.size.width as usize + 7) / 8;

        position += self.bounding_box.top_left;

        for dy in 0..self.bounding_box.size.height {
            for dx in 0..self.bounding_box.size.width {
                let byte_index = dy as usize * bytes_per_row + dx as usize / 8;

                let byte = self.data[byte_index];
                let mask = 0x80 >> dx % 8;

                if byte & mask != 0 {
                    let point = Point::new(dx as i32, dy as i32) + position;
                    Pixel(point, color).draw(target)?;
                }
            }
        }

        Ok(())
    }
}

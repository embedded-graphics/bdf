use embedded_graphics::{
    prelude::*,
    primitives::Rectangle,
    text::{CharacterStyle, TextMetrics, TextRenderer, VerticalAlignment},
};

use crate::BdfFont;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BdfTextStyle<'a, 'b, 'c, C> {
    font: &'a BdfFont<'b, 'c>,
    color: C,
}

impl<'a, 'b, 'c, C> BdfTextStyle<'a, 'b, 'c, C>
where
    C: PixelColor,
{
    pub fn new(font: &'a BdfFont<'b, 'c>, color: C) -> Self {
        Self { font, color }
    }
}

impl<'a, 'b, 'c, C> CharacterStyle for BdfTextStyle<'a, 'b, 'c, C>
where
    C: PixelColor,
{
    type Color = C;

    fn set_text_color(&mut self, text_color: Option<Self::Color>) {
        // TODO: support transparent text
        if let Some(color) = text_color {
            self.color = color;
        }
    }

    // TODO: implement additional methods
}

impl<'a, 'b, 'c, C> TextRenderer for BdfTextStyle<'a, 'b, 'c, C>
where
    C: PixelColor,
{
    type Color = C;

    fn draw_string<D>(
        &self,
        text: &str,
        mut position: Point,
        target: &mut D,
    ) -> Result<Point, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        for c in text.chars() {
            if let Some(glyph) = self.font.get_glyph(c) {
                glyph.draw(position, self.color, target)?;

                position.x += glyph.device_width as i32;
            } else {
                //TODO: how should missing glyphs be handled?
            }
        }

        Ok(position)
    }

    fn draw_whitespace<D>(
        &self,
        width: u32,
        position: Point,
        _target: &mut D,
    ) -> Result<Point, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        Ok(position + Size::new(width, 0))
    }

    fn measure_string(&self, text: &str, position: Point) -> TextMetrics {
        // TODO: handle missing glyphs the same way as `draw_string` does
        let dx = text
            .chars()
            .filter_map(|c| self.font.get_glyph(c))
            .map(|glyph| glyph.device_width)
            .sum();

        // TODO: calculate bounding box
        TextMetrics {
            bounding_box: Rectangle::new(position, Size::zero()),
            next_position: position + Size::new(dx, 0),
        }
    }

    fn vertical_offset(&self, position: Point, _vertical_alignment: VerticalAlignment) -> Point {
        // TODO: support other alignments
        position
    }

    fn line_height(&self) -> u32 {
        // TODO: read line height from BDF file
        11
    }
}

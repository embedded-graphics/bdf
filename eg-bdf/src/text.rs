use embedded_graphics::{
    prelude::*,
    primitives::Rectangle,
    text::{
        renderer::{CharacterStyle, TextMetrics, TextRenderer},
        Baseline,
    },
};

use crate::BdfFont;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BdfTextStyle<'a, C> {
    font: &'a BdfFont<'a>,
    color: C,
}

impl<'a, C: PixelColor> BdfTextStyle<'a, C> {
    pub fn new(font: &'a BdfFont<'a>, color: C) -> Self {
        Self { font, color }
    }
}

impl<C: PixelColor> CharacterStyle for BdfTextStyle<'_, C> {
    type Color = C;

    fn set_text_color(&mut self, text_color: Option<Self::Color>) {
        // TODO: support transparent text
        if let Some(color) = text_color {
            self.color = color;
        }
    }

    // TODO: implement additional methods
}

impl<C: PixelColor> TextRenderer for BdfTextStyle<'_, C> {
    type Color = C;

    fn draw_string<D>(
        &self,
        text: &str,
        mut position: Point,
        _baseline: Baseline,
        target: &mut D,
    ) -> Result<Point, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        // TODO: handle baseline

        for c in text.chars() {
            let glyph = self.font.get_glyph(c);

            glyph.draw(position, self.color, self.font.data, target)?;

            position.x += glyph.device_width as i32;
        }

        Ok(position)
    }

    fn draw_whitespace<D>(
        &self,
        width: u32,
        position: Point,
        _baseline: Baseline,
        _target: &mut D,
    ) -> Result<Point, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        // TODO: handle baseline

        Ok(position + Size::new(width, 0))
    }

    fn measure_string(&self, text: &str, position: Point, _baseline: Baseline) -> TextMetrics {
        // TODO: handle baseline
        let dx = text
            .chars()
            .map(|c| self.font.get_glyph(c).device_width)
            .sum();

        // TODO: calculate bounding box
        TextMetrics {
            bounding_box: Rectangle::new(position, Size::zero()),
            next_position: position + Size::new(dx, 0),
        }
    }

    fn line_height(&self) -> u32 {
        self.font.line_height
    }
}

use embedded_graphics::{
    prelude::*,
    primitives::Rectangle,
    text::{
        renderer::{CharacterStyle, TextMetrics, TextRenderer},
        Baseline,
    },
};

use crate::BdfFont;

/// BDF character style.
// TODO: rename to character style?
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BdfTextStyle<'a, C> {
    font: &'a BdfFont<'a>,
    color: C,
}

impl<'a, C: PixelColor> BdfTextStyle<'a, C> {
    /// Creates a new character style.
    pub fn new(font: &'a BdfFont<'a>, color: C) -> Self {
        Self { font, color }
    }

    fn baseline_offset(&self, baseline: Baseline) -> i32 {
        match baseline {
            Baseline::Top => self.font.ascent.saturating_sub(1) as i32,
            Baseline::Bottom => -(self.font.descent as i32),
            Baseline::Middle => (self.font.ascent as i32 - self.font.descent as i32) / 2,
            Baseline::Alphabetic => 0,
        }
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
        position: Point,
        baseline: Baseline,
        target: &mut D,
    ) -> Result<Point, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let mut position = position + Point::new(0, self.baseline_offset(baseline));

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
        baseline: Baseline,
        _target: &mut D,
    ) -> Result<Point, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let position = position + Point::new(0, self.baseline_offset(baseline));

        Ok(position + Size::new(width, 0))
    }

    fn measure_string(&self, text: &str, position: Point, baseline: Baseline) -> TextMetrics {
        let position = position + Point::new(0, self.baseline_offset(baseline));

        let dx = text
            .chars()
            .map(|c| self.font.get_glyph(c).device_width)
            .sum();

        // TODO: calculate correct bounding box
        let bounding_box = Rectangle::new(
            position - Size::new(0, self.font.ascent.saturating_sub(1)),
            Size::new(dx, self.line_height()),
        );

        TextMetrics {
            bounding_box,
            next_position: position + Size::new(dx, 0),
        }
    }

    fn line_height(&self) -> u32 {
        // TODO: add separate line height field?
        self.font.ascent + self.font.descent
    }
}

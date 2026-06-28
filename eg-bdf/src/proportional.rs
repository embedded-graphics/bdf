use crate::{BdfFont, DisplayBdfGlyph};
use embedded_graphics::{
    prelude::*,
    primitives::Rectangle,
    text::{
        renderer::{CharacterStyle, TextMetrics, TextRenderer},
        Baseline,
    },
};

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct Metrics {
    pub ascent: u32,
    pub descent: u32,
    pub line_height: u32,
}

/// A proportional font
pub trait ProportionalFont<'a>: Clone {
    /// Returns a struct containing ascent, descent, baseline_offset, and line_height
    fn metrics(&self) -> Metrics;
    /// Finds a BdfGlyph for a character
    fn lookup(&self, c: char) -> Option<DisplayBdfGlyph<'_>>;
    /// Finds the replacement glyph
    fn replacement_glyph(&'a self) -> DisplayBdfGlyph<'a>;

    /// Returns the baseline offset
    fn baseline_offset(&self, baseline: Baseline) -> i32 {
        match baseline {
            Baseline::Top => self.metrics().ascent.saturating_sub(1) as i32,
            Baseline::Bottom => -(self.metrics().descent as i32),
            Baseline::Middle => (self.metrics().ascent as i32 - self.metrics().descent as i32) / 2,
            Baseline::Alphabetic => 0,
        }
    }

    /// Returns a glyph, or a replacement character if no corresponding glyph exists
    fn glyph_or_replacement(&'a self, c: char) -> DisplayBdfGlyph<'a> {
        self.lookup(c).unwrap_or(self.replacement_glyph())
    }
}

impl<'a> ProportionalFont<'a> for BdfFont<'a> {
    fn metrics(&self) -> Metrics {
        Metrics {
            ascent: self.ascent,
            descent: self.descent,
            line_height: self.ascent + self.descent,
        }
    }

    fn replacement_glyph(&'a self) -> DisplayBdfGlyph<'a> {
        self.glyphs[self.replacement_character].into_glyph(self)
    }

    fn lookup(&self, c: char) -> Option<DisplayBdfGlyph<'_>> {
        if let Some(&g) = self.glyphs.iter().find(|g| g.character == c) {
            Some(g.into_glyph(self))
        } else {
            None
        }
    }
}

/// A generalized text style for proportional fonts
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProportionalTextStyle<'a, F: ProportionalFont<'a>, C: PixelColor> {
    font: &'a F,
    color: C,
}

impl<'a, F: ProportionalFont<'a>, C: PixelColor> ProportionalTextStyle<'a, F, C> {
    /// Creates a new text style
    pub fn new(font: &'a F, color: C) -> Self {
        Self { font, color }
    }
}

impl<'a, C: PixelColor, F: ProportionalFont<'a>> CharacterStyle
    for ProportionalTextStyle<'a, F, C>
{
    type Color = C;

    fn set_text_color(&mut self, text_color: Option<C>) {
        // TODO: support transparent text
        if let Some(color) = text_color {
            self.color = color;
        }
    }

    // TODO: implement additional methods
}

impl<'a, C: PixelColor, F: ProportionalFont<'a>> TextRenderer for ProportionalTextStyle<'a, F, C> {
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
        let mut position = position + Point::new(0, self.font.baseline_offset(baseline));

        for c in text.chars() {
            let glyph = self.font.glyph_or_replacement(c);

            glyph.draw(position, self.color, target)?;

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
        let position = position + Point::new(0, self.font.baseline_offset(baseline));

        Ok(position + Size::new(width, 0))
    }

    fn measure_string(&self, text: &str, position: Point, baseline: Baseline) -> TextMetrics {
        let position = position + Point::new(0, self.font.baseline_offset(baseline));

        let dx = text
            .chars()
            .map(|c| self.font.glyph_or_replacement(c).device_width)
            .sum();

        // TODO: calculate correct bounding box
        let bounding_box = Rectangle::new(
            position - Size::new(0, self.font.metrics().ascent.saturating_sub(1)),
            Size::new(dx, self.font.metrics().line_height),
        );

        TextMetrics {
            bounding_box,
            next_position: position + Size::new(dx, 0),
        }
    }

    fn line_height(&self) -> u32 {
        self.font.metrics().line_height
    }
}

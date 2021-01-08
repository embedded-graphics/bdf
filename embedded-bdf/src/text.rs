use embedded_graphics::{fonts::Text, prelude::*};

use crate::BdfFont;

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

pub struct BdfStyled<'a, 'b, 'c, 'd, C> {
    text: Text<'a>,
    style: BdfTextStyle<'b, 'c, 'd, C>,
}

impl<'a, 'b, 'c, 'd, C> BdfStyled<'a, 'b, 'c, 'd, C> {
    pub fn new(text: Text<'a>, style: BdfTextStyle<'b, 'c, 'd, C>) -> Self {
        Self { text, style }
    }
}

impl<C> Drawable for BdfStyled<'_, '_, '_, '_, C>
where
    C: PixelColor,
{
    type Color = C;

    fn draw<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        self.style
            .font
            .draw(self.text.text, self.text.position, self.style.color, target)
    }
}

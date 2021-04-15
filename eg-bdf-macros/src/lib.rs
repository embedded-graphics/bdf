use bdf_parser::{BdfFont, BoundingBox, Glyph, Property};
use embedded_graphics::{prelude::*, primitives::Rectangle};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::{convert::TryFrom, fs, path::PathBuf};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    LitChar, LitStr, Result, Token,
};

struct IncludeBdf {
    filename: LitStr,
    character_ranges: Option<CharacterRanges>,
}

impl IncludeBdf {
    fn contains(&self, c: char) -> bool {
        self.character_ranges
            .as_ref()
            .map(|ranges| ranges.contains(c))
            .unwrap_or(true)
    }
}

impl Parse for IncludeBdf {
    fn parse(input: ParseStream) -> Result<Self> {
        let filename = input.parse()?;

        let character_ranges = if input.lookahead1().peek(Token![,]) {
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self {
            filename,
            character_ranges,
        })
    }
}

struct CharacterRanges {
    ranges: Punctuated<CharacterRange, Token![|]>,
}

impl CharacterRanges {
    fn contains(&self, c: char) -> bool {
        for range in self.ranges.iter() {
            if range.contains(c) {
                return true;
            }
        }

        false
    }
}

impl Parse for CharacterRanges {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Token![,]>()?;

        Ok(Self {
            ranges: Punctuated::parse_separated_nonempty(input)?,
        })
    }
}

struct CharacterRange {
    from: LitChar,
    to: Option<(Token![..=], LitChar)>,
}

impl CharacterRange {
    fn contains(&self, c: char) -> bool {
        match &self.to {
            None => c == self.from.value(),
            Some((_, to)) => (self.from.value()..=to.value()).contains(&c),
        }
    }
}

impl Parse for CharacterRange {
    fn parse(input: ParseStream) -> Result<Self> {
        let from = input.parse()?;
        let to = if input.lookahead1().peek(Token![..=]) {
            Some((input.parse()?, input.parse()?))
        } else {
            None
        };

        Ok(Self { from, to })
    }
}

/// Converts a BDF bounding box into an embedded-graphics rectangle.
fn bounding_box_to_rectangle(bounding_box: &BoundingBox) -> Rectangle {
    Rectangle::new(
        Point::new(
            bounding_box.offset.x,
            -bounding_box.offset.y - (bounding_box.size.y as i32 - 1),
        ),
        // TODO: check for negative values
        Size::new(bounding_box.size.x as u32, bounding_box.size.y as u32),
    )
}

fn rectangle_constructor(rectangle: &Rectangle) -> proc_macro2::TokenStream {
    let Rectangle {
        top_left: Point { x, y },
        size: Size { width, height },
    } = rectangle;

    quote! {
        ::embedded_graphics::primitives::Rectangle::new(
            ::embedded_graphics::geometry::Point::new(#x, #y),
            ::embedded_graphics::geometry::Size::new(#width, #height),
        )
    }
}

fn glyph_literal(glyph: &Glyph, start_index: usize) -> (Vec<bool>, proc_macro2::TokenStream) {
    let character = LitChar::new(glyph.encoding.unwrap(), Span::call_site());

    let rectangle = bounding_box_to_rectangle(&glyph.bounding_box);
    let bounding_box = rectangle_constructor(&rectangle);

    // TODO: handle height != 0
    // TODO: check for negative values
    let device_width = glyph.device_width.x as u32;

    // let bitmap = &glyph.bitmap;
    // let data = quote! { &[ #( #bitmap ),* ] };
    let mut data = Vec::new();

    for y in 0..usize::try_from(glyph.bounding_box.size.y).unwrap() {
        for x in 0..usize::try_from(glyph.bounding_box.size.x).unwrap() {
            data.push(glyph.pixel(x, y))
        }
    }

    (
        data,
        quote! {
            ::eg_bdf::BdfGlyph {
                character: #character,
                bounding_box: #bounding_box,
                device_width: #device_width,
                start_index: #start_index,
            }
        },
    )
}

#[proc_macro]
pub fn include_bdf(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as IncludeBdf);

    // TODO: handle errors
    let mut path = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    path.push(&input.filename.value());

    // TODO: handle errors
    let bdf = fs::read(&path).unwrap();

    let font = BdfFont::parse(&bdf).unwrap();

    let mut data = Vec::new();
    let mut glyphs = Vec::new();
    let mut replacement_character = None;

    //TODO: sort glyphs to make it possible to use binary search
    for glyph in font.glyphs.iter() {
        if let Some(c) = glyph.encoding {
            if !input.contains(c) {
                continue;
            }

            if c == std::char::REPLACEMENT_CHARACTER {
                replacement_character = Some(glyphs.len());
            } else if c == ' ' && replacement_character == None {
                replacement_character = Some(glyphs.len());
            } 

            let (glyph_data, literal) = glyph_literal(glyph, data.len());
            glyphs.push(literal);
            data.extend_from_slice(&glyph_data);
        }
    }

    // TODO: try to use DEFAULT_CHAR
    let replacement_character = replacement_character.unwrap_or_default();

    let data = bits_to_bytes(&data);

    // TODO: report error or calculate fallback value
    let line_height = font.properties.try_get::<i32>(Property::PixelSize).unwrap() as u32;

    let output = quote! {
        ::eg_bdf::BdfFont {
            glyphs: &[ #( #glyphs ),* ],
            data: &[ #( #data ),* ],
            line_height: #line_height,
            replacement_character: #replacement_character,
        }
    };

    output.into()
}

fn bits_to_bytes(bits: &[bool]) -> Vec<u8> {
    bits.chunks(8)
        .map(|bits| {
            bits.iter()
                .enumerate()
                .filter(|(_, b)| **b)
                .map(|(i, _)| 0x80 >> i)
                .sum()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bits_to_bytes() {
        let f = false;
        let t = true;

        assert_eq!(bits_to_bytes(&[f, f, f, f, f, f, f, f]), vec![0x00]);
        assert_eq!(bits_to_bytes(&[t, f, f, f, f, f, f, f]), vec![0x80]);
        assert_eq!(bits_to_bytes(&[t, f, f, f, f, f, f, t]), vec![0x81]);
    }

    #[test]
    fn test_bits_to_bytes_incomplete_byte() {
        let f = false;
        let t = true;

        assert_eq!(
            bits_to_bytes(&[f, f, f, f, f, f, f, f, t]),
            vec![0x00, 0x80]
        );
    }
}

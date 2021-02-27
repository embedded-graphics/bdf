use bdf_parser::{BdfFont, BoundingBox, Glyph};
use embedded_graphics::{prelude::*, primitives::Rectangle};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::{fs, path::PathBuf};
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

fn glyph_literal(glyph: &Glyph) -> Option<proc_macro2::TokenStream> {
    let character = LitChar::new(glyph.encoding?, Span::call_site());

    let rectangle = bounding_box_to_rectangle(&glyph.bounding_box);
    let bounding_box = rectangle_constructor(&rectangle);

    // TODO: handle height != 0
    // TODO: check for negative values
    let device_width = glyph.device_width.x as u32;

    let bitmap = &glyph.bitmap;
    let data = quote! { &[ #( #bitmap ),* ] };

    Some(quote! {
        ::eg_bdf::BdfGlyph {
            character: #character,
            bounding_box: #bounding_box,
            device_width: #device_width,
            data: #data,
        }
    })
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

    //TODO: sort glyphs to make it possible to use binary search
    let glyphs: Vec<_> = font
        .glyphs
        .iter()
        .filter(|glyph| glyph.encoding.map(|c| input.contains(c)).unwrap_or(false))
        .filter_map(glyph_literal)
        .collect();

    let output = quote! {
        ::eg_bdf::BdfFont {
            glyphs: &[ #( #glyphs ),* ]
        }
    };

    output.into()
}

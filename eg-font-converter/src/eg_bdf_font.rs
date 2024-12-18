use std::{fs, io, path::Path};

use anyhow::Result;
use bdf_parser::BoundingBox;
use bitvec::{prelude::*, vec::BitVec};
use eg_bdf::{BdfFont, BdfGlyph};
use embedded_graphics::{
    geometry::{Point, Size},
    primitives::Rectangle,
};
use quote::{format_ident, quote};

use crate::Font;

/// Converts a BDF bounding box into an embedded-graphics rectangle.
pub fn bounding_box_to_rectangle(bounding_box: &BoundingBox) -> Rectangle {
    Rectangle::new(
        Point::new(
            bounding_box.offset.x,
            -bounding_box.offset.y - (bounding_box.size.y - 1),
        ),
        // TODO: check for negative values
        Size::new(bounding_box.size.x as u32, bounding_box.size.y as u32),
    )
}

/// Font conversion output for the [`eg-bdf`] crate.
///
/// [`eg-bdf`]: eg_bdf
#[derive(Debug)]
pub struct EgBdfOutput {
    pub(crate) font: Font,
    data: BitVec<u8, Msb0>,
    glyphs: Vec<BdfGlyph>,
}

impl EgBdfOutput {
    pub(crate) fn new(font: Font) -> Result<Self> {
        let mut data = BitVec::<u8, Msb0>::new();
        let mut glyphs = Vec::new();

        for glyph in font.glyphs.iter() {
            let bounding_box = bounding_box_to_rectangle(&glyph.bounding_box);

            glyphs.push(BdfGlyph {
                character: glyph.encoding.unwrap(),
                bounding_box,
                device_width: glyph.device_width.x as u32, // TODO: check cast and handle y?
                start_index: data.len(),
            });

            data.extend(glyph.pixels());
        }

        Ok(Self { font, data, glyphs })
    }

    /// Returns the generated Rust code.
    pub fn rust(&self) -> String {
        self.try_rust().unwrap()
    }

    fn try_rust(&self) -> Result<String> {
        let constant_name = format_ident!("{}", self.font.name);
        let data_file = self.font.data_file().to_string_lossy().to_string();
        let Font {
            replacement_character,
            ascent,
            descent,
            ..
        } = self.font;

        let glyphs = self.glyphs.iter().map(|glyph| {
            let BdfGlyph {
                character,
                bounding_box:
                    Rectangle {
                        top_left: Point { x, y },
                        size: Size { width, height },
                    },
                device_width,
                start_index,
            } = glyph;

            quote!(::eg_bdf::BdfGlyph {
                character: #character,
                bounding_box: rect(#x, #y, #width, #height),
                device_width: #device_width,
                start_index: #start_index,
            })
        });

        let comments = self.font.comments.iter().map(|comment| {
            let comment = format!(" {}", comment);
            quote!(
                #[doc = #comment]
            )
        });

        Ok(prettyplease::unparse(&syn::parse2(quote!(
            #( #comments )*
            const #constant_name: ::eg_bdf::BdfFont = {
                const fn rect(x: i32, y: i32, width: u32, height: u32) -> ::embedded_graphics::primitives::Rectangle {
                    ::embedded_graphics::primitives::Rectangle::new(
                        ::embedded_graphics::geometry::Point::new(x, y),
                        ::embedded_graphics::geometry::Size::new(width, height),
                    )
                }

                ::eg_bdf::BdfFont {
                    data: include_bytes!(#data_file),
                    replacement_character: #replacement_character,
                    ascent: #ascent,
                    descent: #descent,
                    glyphs: &[ #(  #glyphs , )* ],
                }
            };
        ))?))
    }

    /// Returns the bitmap data.
    pub fn data(&self) -> &[u8] {
        self.data.as_raw_slice()
    }

    /// Returns the converted font as a [`BdfFont`].
    pub fn as_font(&self) -> BdfFont<'_> {
        BdfFont {
            replacement_character: self.font.replacement_character,
            ascent: self.font.ascent,
            descent: self.font.descent,
            glyphs: &self.glyphs,
            data: self.data(),
        }
    }

    /// Saves the rust file and bitmap data to the given directory.
    pub fn save<P: AsRef<Path>>(&self, output_directory: P) -> io::Result<()> {
        let output_directory = output_directory.as_ref();

        fs::write(self.font.rust_file_path(output_directory), &self.rust())?;
        fs::write(self.font.data_file_path(output_directory), &self.data())?;

        Ok(())
    }
}

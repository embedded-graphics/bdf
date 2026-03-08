use std::{fs, io, ops::RangeInclusive, path::Path};

use anyhow::{bail, ensure, Context, Result};
use bdf_parser::{BoundingBox, Coord, Encoding, Font, Glyph};
use eg_bdf::BdfTextStyle;
use embedded_graphics::{
    image::ImageRaw,
    mono_font::{mapping::Mapping, DecorationDimensions, MonoFont},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::Rectangle,
    text::Text,
};
use embedded_graphics_simulator::{OutputSettings, SimulatorDisplay};
use quote::{format_ident, quote};

use crate::{eg_bdf_font::bounding_box_to_rectangle, ConvertedFont, EgBdfOutput, GlyphAlignment};

/// Font conversion output for [`MonoFont`].
#[derive(Debug)]
pub struct MonoFontOutput {
    font: ConvertedFont,
    bitmap: SimulatorDisplay<BinaryColor>,
    data: Vec<u8>,

    character_size: Size,
    character_spacing: u32,
    baseline: u32,
    underline: DecorationDimensions,
    strikethrough: DecorationDimensions,

    glyphs: Vec<Glyph>,
    mapping: Option<Mapping>,
}

fn determine_glyph_metrics<'a>(glyphs: impl Iterator<Item = &'a Glyph>) -> (Rectangle, u32) {
    // Default to 1x1 pixel bounding box at the origin.
    // This ensures that the relative positioning of glyphs that don't touch the baseline is
    // maintained in case only this kind of glyph is included.
    let mut bounding_box = BoundingBox {
        offset: Coord::new(0, 0),
        size: Coord::new(1, 1),
    };

    let mut max_dwidth = 0;
    for glyph in glyphs {
        bounding_box = bounding_box.union(&glyph.bounding_box);
        if let Some(width) = glyph.width_horizontal {
            max_dwidth = max_dwidth.max(width.device.x);
        }
    }

    (
        bounding_box_to_rectangle(&bounding_box),
        max_dwidth.try_into().unwrap(),
    )
}

impl MonoFontOutput {
    pub(crate) fn new(bdf: EgBdfOutput) -> Result<Self> {
        let font = bdf.as_font();
        let style = BdfTextStyle::new(&font, BinaryColor::On);

        let glyphs_per_row = 16; //TODO: make configurable
        let columns = glyphs_per_row; // TODO: allow smaller column count
        let rows = font.glyphs.len().div_ceil(glyphs_per_row);

        let mapping = glyphs_to_mapping(&bdf.font.glyphs);

        // Rearrange the glyphs in the correct order if a mapping is used,
        // because e-g monofont mappings aren't sorted by the Unicode codepoint.
        let glyphs = if let Some(mapping) = mapping {
            mapping
                .glyph_mapping()
                .chars()
                .map(|c| {
                    let index = bdf.font.glyph_index(c).unwrap();
                    bdf.font.glyphs[index].clone()
                })
                .collect::<Vec<_>>()
        } else {
            // Filter out all non standard glyphs.
            // TODO: add a warning
            bdf.font
                .glyphs
                .iter()
                .filter(|g| matches!(g.encoding, Encoding::Standard(_)))
                .cloned()
                .collect()
        };

        ensure!(!glyphs.is_empty(), "Font must contain at least one glyph");

        let (bounding_box, max_dwidth) = determine_glyph_metrics(glyphs.iter());

        let character_size = bounding_box.size;
        let character_spacing = max_dwidth.saturating_sub(character_size.width);

        let baseline: u32 = (-bounding_box.top_left.y).try_into().unwrap();
        let strikethrough = DecorationDimensions::new(
            bdf.font.strikethrough_position,
            bdf.font.strikethrough_thickness,
        );
        let underline =
            DecorationDimensions::new(bdf.font.underline_position, bdf.font.underline_thickness);

        let mut bitmap = SimulatorDisplay::new(
            character_size.component_mul(Size::new(columns as u32, rows as u32)),
        );

        for (i, glyph) in glyphs.iter().enumerate() {
            // TODO: assumes unicode
            let c = match glyph.encoding {
                Encoding::Standard(index) => char::from_u32(index).unwrap(),
                _ => bail!("invalid encoding: '{:?}'", glyph.encoding),
            };

            let (row, column) = (i / glyphs_per_row, i % glyphs_per_row);

            let glyph_position =
                Point::zero() + Size::new(column as u32, row as u32).component_mul(character_size);
            let baseline_offset = Point::new(-glyph.bounding_box.offset.x, baseline as i32);
            let alignment_offset = match bdf.font.glyph_alignment {
                GlyphAlignment::Left => 0,
                GlyphAlignment::Center => {
                    (character_size.width as i32 - glyph.bounding_box.size.x) / 2
                }
                GlyphAlignment::Right => character_size.width as i32 - glyph.bounding_box.size.x,
            };

            Text::new(
                &String::from(c),
                glyph_position + baseline_offset + Point::new(alignment_offset, 0),
                style,
            )
            .draw(&mut bitmap)
            .unwrap();
        }

        let data = bitmap.to_be_bytes();

        Ok(Self {
            font: bdf.font,
            bitmap,
            data,
            character_size,
            character_spacing,
            baseline,
            underline,
            strikethrough,
            glyphs,
            mapping,
        })
    }

    /// Returns the rust code.
    pub fn rust(&self) -> String {
        self.try_rust().unwrap()
    }

    fn try_rust(&self) -> Result<String> {
        let const_name = format_ident!("{}", self.font.name);
        let image_data = self.font.data_file().to_string_lossy().to_string();
        let image_width = self.bitmap.size().width;
        let MonoFont {
            character_size:
                Size {
                    width: character_width,
                    height: character_height,
                },
            character_spacing,
            baseline,
            underline:
                DecorationDimensions {
                    offset: underline_offset,
                    height: underline_height,
                },
            strikethrough:
                DecorationDimensions {
                    offset: strikethrough_offset,
                    height: strikethrough_height,
                },
            ..
        } = self.as_font();

        let glyph_mapping = if let Some(mapping) = self.mapping {
            let mime = format_ident!("{}", mapping.mime());
            quote!(::embedded_graphics::mono_font::mapping::#mime)
        } else {
            let str_mapping = glyphs_to_str_mapping(self.glyphs.iter().map(|glyph| {
                // TODO: assumes unicode
                match glyph.encoding {
                    Encoding::Standard(index) => char::from_u32(index).unwrap(),
                    _ => unreachable!(),
                }
            }));
            let replacement = self.font.replacement_character;
            quote!(::embedded_graphics::mono_font::mapping::StrGlyphMapping::new(#str_mapping, #replacement))
        };

        let comments = self.font.comments.iter().map(|comment| {
            let comment = format!(" {comment}");
            quote!(
                #[doc = #comment]
            )
        });

        Ok(prettyplease::unparse(&syn::parse2(quote!(
            #( #comments )*
            pub const #const_name: ::embedded_graphics::mono_font::MonoFont = ::embedded_graphics::mono_font::MonoFont {
                image: ::embedded_graphics::image::ImageRaw::new(include_bytes!(#image_data), #image_width),
                glyph_mapping: &#glyph_mapping,
                character_size: ::embedded_graphics::geometry::Size::new(#character_width, #character_height),
                character_spacing: #character_spacing,
                baseline: #baseline,
                underline: ::embedded_graphics::mono_font::DecorationDimensions::new(#underline_offset, #underline_height),
                strikethrough: ::embedded_graphics::mono_font::DecorationDimensions::new(#strikethrough_offset, #strikethrough_height),
            };
        ))?))
    }

    /// Returns the bitmap data.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Returns the conversion result as a [`MonoFont`].
    pub fn as_font(&self) -> MonoFont<'_> {
        let image = ImageRaw::new(self.data(), self.bitmap.size().width);

        MonoFont {
            image,
            character_size: self.character_size,
            character_spacing: self.character_spacing,
            baseline: self.baseline,
            strikethrough: self.strikethrough,
            underline: self.underline,
            glyph_mapping: &self.font,
        }
    }

    /// Saves the rust code and bitmap data to a directory.
    pub fn save<P: AsRef<Path>>(&self, output_directory: P) -> io::Result<()> {
        let output_directory = output_directory.as_ref();

        fs::write(self.font.rust_file_path(output_directory), self.rust())?;
        fs::write(self.font.data_file_path(output_directory), self.data())?;

        Ok(())
    }

    /// Returns the BDF file.
    pub fn bdf(&self) -> &Font {
        &self.font.bdf
    }

    /// Saves the generated bitmap as a PNG file.
    pub fn save_png<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();

        self.bitmap
            .to_grayscale_output_image(&OutputSettings::default())
            .save_png(path)
            .with_context(|| format!("failed to write PNG file to {}", path.display()))
    }
}

fn glyphs_to_str_mapping(glyphs: impl Iterator<Item = char>) -> String {
    let mut ranges: Vec<RangeInclusive<u32>> = Vec::new();

    for c in glyphs.map(|c| c as u32) {
        match ranges.last_mut() {
            Some(range) if c == *range.end() + 1 => {
                *range = *range.start()..=c;
            }
            _ => ranges.push(c..=c),
        }
    }

    let mut mapping = String::new();
    for range in ranges {
        assert!(!range.is_empty());

        let chars = range.end() - range.start() + 1;
        if chars == 1 {
            mapping.push(char::from_u32(*range.start()).unwrap());
        } else {
            if chars > 2 {
                mapping.push('\0');
            }
            mapping.push(char::from_u32(*range.start()).unwrap());
            mapping.push(char::from_u32(*range.end()).unwrap());
        }
    }

    mapping
}

// TODO: also check the replacement character
fn glyphs_to_mapping(glyphs: &[Glyph]) -> Option<Mapping> {
    for mapping in Mapping::iter() {
        let mut chars = mapping.glyph_mapping().chars().collect::<Vec<_>>();
        chars.sort();

        if glyphs
            .iter()
            .map(|glyph| {
                // TODO: assumes unicode
                match glyph.encoding {
                    Encoding::Standard(index) => char::from_u32(index),
                    _ => None,
                }
            })
            .eq(chars.iter().map(|c| Some(*c)))
        {
            return Some(mapping);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn test_glyphs_to_str_mapping() {
        let mut glyphs = BTreeSet::new();
        assert_eq!(glyphs_to_str_mapping(glyphs.iter().copied()), "");

        glyphs.insert('a');
        assert_eq!(glyphs_to_str_mapping(glyphs.iter().copied()), "a");

        glyphs.insert('b');
        assert_eq!(glyphs_to_str_mapping(glyphs.iter().copied()), "ab");

        glyphs.insert('c');
        assert_eq!(glyphs_to_str_mapping(glyphs.iter().copied()), "\0ac");

        glyphs.insert('d');
        assert_eq!(glyphs_to_str_mapping(glyphs.iter().copied()), "\0ad");

        glyphs.insert('x');
        assert_eq!(glyphs_to_str_mapping(glyphs.iter().copied()), "\0adx");

        glyphs.insert('y');
        assert_eq!(glyphs_to_str_mapping(glyphs.iter().copied()), "\0adxy");

        glyphs.insert('z');
        assert_eq!(glyphs_to_str_mapping(glyphs.iter().copied()), "\0ad\0xz");
    }

    #[test]
    fn test_glyphs_to_mapping() {
        let glyphs = (' '..='\x7F')
            .map(|c| Glyph {
                encoding: Encoding::Standard(c as u32),
                ..Glyph::default()
            })
            .collect::<Vec<_>>();
        assert_eq!(glyphs_to_mapping(&glyphs), Some(Mapping::Ascii));

        let glyphs = (' '..='\x7F')
            .chain(0xA0 as char..=0xFF as char)
            .map(|c| Glyph {
                encoding: Encoding::Standard(c as u32),
                ..Glyph::default()
            })
            .collect::<Vec<_>>();
        assert_eq!(glyphs_to_mapping(&glyphs), Some(Mapping::Iso8859_1));
    }
}

use std::{fs, io, ops::RangeInclusive, path::Path};

use anyhow::{bail, Context, Result};
use bdf_parser::{BdfFont as ParserBdfFont, Encoding, Glyph};
use eg_bdf::BdfTextStyle;
use embedded_graphics::{
    image::ImageRaw,
    mono_font::{mapping::Mapping, DecorationDimensions, MonoFont},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use embedded_graphics_simulator::{OutputSettings, SimulatorDisplay};
use quote::{format_ident, quote};

use crate::{EgBdfOutput, Font};

/// Font conversion output for [`MonoFont`].
#[derive(Debug)]
pub struct MonoFontOutput {
    font: Font,
    bitmap: SimulatorDisplay<BinaryColor>,
    data: Vec<u8>,

    character_size: Size,
    character_spacing: u32,
    baseline: u32,
    underline: DecorationDimensions,
    strikethrough: DecorationDimensions,

    mapping: Option<Mapping>,
}

impl MonoFontOutput {
    pub(crate) fn new(bdf: EgBdfOutput) -> Result<Self> {
        let font = bdf.as_font();
        let style = BdfTextStyle::new(&font, BinaryColor::On);

        let glyphs_per_row = 16; //TODO: make configurable
        let columns = glyphs_per_row; // TODO: allow smaller column count
        let rows = (font.glyphs.len() + (glyphs_per_row - 1)) / glyphs_per_row;

        let character_size = bdf.bounding_box().size;
        let character_spacing = 0;
        let baseline = bdf.font.ascent.saturating_sub(1);
        let strikethrough = DecorationDimensions::new(
            bdf.font.strikethrough_position,
            bdf.font.strikethrough_thickness,
        );
        let underline =
            DecorationDimensions::new(bdf.font.underline_position, bdf.font.underline_thickness);

        let mut bitmap = SimulatorDisplay::new(
            character_size.component_mul(Size::new(columns as u32, rows as u32)),
        );

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
            bdf.font.glyphs.clone()
        };

        for (i, glyph) in glyphs.iter().enumerate() {
            let x = (i % glyphs_per_row) as i32 * character_size.width as i32;
            let y = (i / glyphs_per_row) as i32 * character_size.height as i32;

            // TODO: assumes unicode
            let c = match glyph.encoding {
                Encoding::Standard(index) => char::from_u32(index).unwrap(),
                _ => bail!("invalid encoding"),
            };

            Text::with_baseline(&String::from(c), Point::new(x, y), style, Baseline::Top)
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
            let str_mapping = glyphs_to_str_mapping(self.font.glyphs.iter().map(|glyph| {
                // TODO: assumes unicode
                let c = match glyph.encoding {
                    Encoding::Standard(index) => char::from_u32(index).unwrap(),
                    _ => panic!("invalid encoding"),
                };

                c
            }));
            let replacement = self.font.replacement_character;
            quote!(::embedded_graphics::mono_font::mapping::StrGlyphMapping::new(#str_mapping, #replacement))
        };

        let comments = self.font.comments.iter().map(|comment| {
            let comment = format!(" {}", comment);
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

        fs::write(self.font.rust_file_path(output_directory), &self.rust())?;
        fs::write(self.font.data_file_path(output_directory), &self.data())?;

        Ok(())
    }

    /// Returns the BDF file.
    pub fn bdf(&self) -> &ParserBdfFont {
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
                let c = match glyph.encoding {
                    Encoding::Standard(index) => char::from_u32(index).unwrap(),
                    _ => panic!("invalid encoding"),
                };

                c
            })
            .eq(chars.iter().copied())
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

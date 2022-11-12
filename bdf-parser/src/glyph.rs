use std::convert::TryFrom;

use crate::{
    parser::{Line, Lines},
    BoundingBox, Coord, ParserError,
};

/// Glyph encoding
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Encoding {
    /// Standard encoding
    Standard(u32),
    /// Non standard encoding
    NonStandard(u32),
    /// Unspecified encoding
    #[default]
    Unspecified,
}

/// Glyph.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Glyph {
    /// Name.
    pub name: String,

    /// Encoding.
    pub encoding: Encoding,

    /// Scalable width.
    pub scalable_width: Option<Coord>,

    /// Device width.
    pub device_width: Coord,

    /// Bounding box.
    pub bounding_box: BoundingBox,

    /// Bitmap data.
    pub bitmap: Vec<u8>,
}

fn parse_bitmap_row(line: &Line<'_>, bitmap: &mut Vec<u8>) -> Result<(), ()> {
    if !line.parameters.is_empty() || line.keyword.len() % 2 != 0 {
        return Err(());
    }

    // Accessing the UTF-8 string by byte and not by char is OK because the
    // hex conversion will fail for non ASCII inputs.
    for hex in line.keyword.as_bytes().chunks_exact(2) {
        let byte = str::from_utf8(hex)
            .ok()
            .and_then(|s| u8::from_str_radix(s, 16).ok())
            .ok_or(())?;
        bitmap.push(byte);
    }

    Ok(())
}

impl Glyph {
    pub(crate) fn parse(mut lines: &mut Lines<'_>) -> Result<Self, crate::ParserError> {
        let mut encoding = Encoding::Unspecified;
        let mut scalable_width = None;
        let mut device_width = Coord::new(0, 0);
        let mut bounding_box = BoundingBox {
            size: Coord::new(0, 0),
            offset: Coord::new(0, 0),
        };

        let start = lines.next().unwrap();
        assert_eq!(start.keyword, "STARTCHAR");
        let name = start.parameters;

        for line in &mut lines {
            match line.keyword {
                "ENCODING" => {
                    encoding = if let Some([index1, index2]) = line.parse_integer_parameters() {
                        if index1 >= 0 || index2 < 0 {
                            return Err(ParserError::with_line("invalid \"ENCODING\"", &line));
                        }

                        Encoding::NonStandard(index2 as u32)
                    } else if let Some([index]) = line.parse_integer_parameters() {
                        if index >= 0 {
                            Encoding::Standard(index as u32)
                        } else {
                            Encoding::Unspecified
                        }
                    } else {
                        return Err(ParserError::with_line("invalid \"ENCODING\"", &line));
                    };
                }
                "SWIDTH" => {
                    scalable_width = Some(
                        Coord::parse(&line)
                            .ok_or_else(|| ParserError::with_line("invalid \"SWIDTH\"", &line))?,
                    );
                }
                "DWIDTH" => {
                    device_width = Coord::parse(&line)
                        .ok_or_else(|| ParserError::with_line("invalid \"DWIDTH\"", &line))?;
                }
                "BBX" => {
                    bounding_box = BoundingBox::parse(&line)
                        .ok_or_else(|| ParserError::with_line("invalid \"BBX\"", &line))?;
                }
                "BITMAP" => {
                    break;
                }
                _ => {
                    return Err(ParserError::with_line(
                        &format!("unknown keyword in glyphs: \"{}\"", line.keyword),
                        &line,
                    ))
                }
            }
        }

        let mut bitmap = Vec::new();
        for line in &mut lines {
            if line.keyword == "ENDCHAR" {
                break;
            }

            parse_bitmap_row(&line, &mut bitmap)
                .map_err(|_| ParserError::with_line("invalid hex data in BITMAP", &line))?;
        }

        Ok(Self {
            name: name.to_string(),
            encoding,
            scalable_width,
            device_width,
            bounding_box,
            bitmap,
        })
    }

    /// Returns a pixel from the bitmap.
    ///
    /// This method doesn't use the BDF coordinate system. The coordinates are relative to the
    /// top left corner of the bounding box and don't take the offset into account. Y coordinates
    /// increase downwards.
    ///
    /// Returns `None` if the coordinates are outside the bitmap.
    pub fn pixel(&self, x: usize, y: usize) -> Option<bool> {
        let width = usize::try_from(self.bounding_box.size.x).unwrap();

        if x >= width {
            return None;
        }

        let bytes_per_row = width.div_ceil(8);
        let byte_offset = x / 8;
        let bit_mask = 0x80 >> (x % 8);

        self.bitmap
            .get(byte_offset + bytes_per_row * y)
            .map(|v| v & bit_mask != 0)
    }

    /// Returns an iterator over the pixels in the glyph bitmap.
    ///
    /// Iteration starts at the top left corner of the bounding box and ends at the bottom right
    /// corner.
    pub fn pixels(&self) -> impl Iterator<Item = bool> + '_ {
        let width = usize::try_from(self.bounding_box.size.x).unwrap();
        let height = usize::try_from(self.bounding_box.size.y).unwrap();

        (0..height).flat_map(move |y| (0..width).map(move |x| self.pixel(x, y).unwrap()))
    }
}

/// Glyphs collection.
#[derive(Debug, Clone, PartialEq)]
pub struct Glyphs {
    glyphs: Vec<Glyph>,
}

impl Glyphs {
    pub(crate) fn parse(lines: &mut Lines<'_>) -> Result<Self, ParserError> {
        let mut glyphs = Vec::new();

        while let Some(line) = lines.next() {
            match line.keyword {
                "CHARS" => {
                    // TODO: handle
                }
                "STARTCHAR" => {
                    lines.backtrack(line);
                    glyphs.push(Glyph::parse(lines)?);
                }
                "ENDFONT" => {
                    break;
                }
                _ => {
                    return Err(ParserError::with_line(
                        &format!("unknown keyword: \"{}\"", line.keyword),
                        &line,
                    ))
                }
            }
        }

        Ok(Self { glyphs })
    }

    /// Gets a glyph by the encoding.
    pub fn get(&self, c: char) -> Option<&Glyph> {
        // TODO: this assumes that the font uses unicode
        let encoding = Encoding::Standard(c as u32);

        self.glyphs
            .binary_search_by_key(&encoding, |glyph| glyph.encoding)
            .map_or(None, |i| Some(&self.glyphs[i]))
    }

    /// Returns `true` if the collection contains the given character.
    pub fn contains(&self, c: char) -> bool {
        self.get(c).is_some()
    }

    /// Returns an iterator over all glyphs.
    pub fn iter(&self) -> impl Iterator<Item = &Glyph> {
        self.glyphs.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[track_caller]
    fn parse_glyph(input: &str) -> Glyph {
        let mut lines = Lines::new(input);
        Glyph::parse(&mut lines).unwrap()
    }

    #[test]
    fn test_parse_bitmap() {
        let prefix = "STARTCHAR 0\nBITMAP\n";
        let suffix = "\nENDCHAR";

        for (input, expected) in [
            ("7e", vec![0x7e]),
            ("ff", vec![0xff]),
            ("CCCC", vec![0xcc, 0xcc]),
            ("ffffffff", vec![0xff, 0xff, 0xff, 0xff]),
            (
                "ffffffff\naaaaaaaa",
                vec![0xff, 0xff, 0xff, 0xff, 0xaa, 0xaa, 0xaa, 0xaa],
            ),
            (
                "ff\nff\nff\nff\naa\naa\naa\naa",
                vec![0xff, 0xff, 0xff, 0xff, 0xaa, 0xaa, 0xaa, 0xaa],
            ),
            (
                "00\n00\n00\n00\n18\n24\n24\n42\n42\n7E\n42\n42\n42\n42\n00\n00",
                vec![
                    0x00, 0x00, 0x00, 0x00, 0x18, 0x24, 0x24, 0x42, 0x42, 0x7e, 0x42, 0x42, 0x42,
                    0x42, 0x00, 0x00,
                ],
            ),
        ] {
            let glyph = parse_glyph(&format!("{prefix}{input}{suffix}"));
            assert_eq!(glyph.bitmap, expected);
        }
    }

    /// Returns test data for a single glyph and the expected parsing result
    fn test_data() -> (&'static str, Glyph) {
        (
            indoc! {r#"
                STARTCHAR ZZZZ
                ENCODING 65
                SWIDTH 500 0
                DWIDTH 8 0
                BBX 8 16 0 -2
                BITMAP
                00
                00
                00
                00
                18
                24
                24
                42
                42
                7E
                42
                42
                42
                42
                00
                00
                ENDCHAR
            "#},
            Glyph {
                name: "ZZZZ".to_string(),
                encoding: Encoding::Standard(65), // 'A'
                bitmap: vec![
                    0x00, 0x00, 0x00, 0x00, 0x18, 0x24, 0x24, 0x42, 0x42, 0x7e, 0x42, 0x42, 0x42,
                    0x42, 0x00, 0x00,
                ],
                bounding_box: BoundingBox {
                    size: Coord::new(8, 16),
                    offset: Coord::new(0, -2),
                },
                scalable_width: Some(Coord::new(500, 0)),
                device_width: Coord::new(8, 0),
            },
        )
    }

    #[test]
    fn parse_single_char() {
        let (chardata, expected_glyph) = test_data();
        assert_eq!(parse_glyph(chardata), expected_glyph);
    }

    #[test]
    fn get_glyph_by_char() {
        let (chardata, expected_glyph) = test_data();

        let mut lines = Lines::new(chardata);

        let glyphs = Glyphs::parse(&mut lines).unwrap();
        assert_eq!(glyphs.get('A'), Some(&expected_glyph));
    }

    #[test]
    fn pixel_getter() {
        let (chardata, _) = test_data();
        let glyph = parse_glyph(chardata);

        let bitmap = (0..16)
            .map(|y| {
                (0..8)
                    .map(|x| if glyph.pixel(x, y).unwrap() { '#' } else { ' ' })
                    .collect::<String>()
            })
            .collect::<Vec<_>>();

        assert_eq!(
            bitmap,
            [
                "        ", //
                "        ", //
                "        ", //
                "        ", //
                "   ##   ", //
                "  #  #  ", //
                "  #  #  ", //
                " #    # ", //
                " #    # ", //
                " ###### ", //
                " #    # ", //
                " #    # ", //
                " #    # ", //
                " #    # ", //
                "        ", //
                "        ", //
            ]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
        );
    }

    #[test]
    fn pixels_iterator() {
        let (chardata, _) = test_data();
        let glyph = parse_glyph(chardata);

        let bitmap = glyph
            .pixels()
            .map(|v| if v { '#' } else { ' ' })
            .collect::<String>();

        assert_eq!(
            bitmap,
            concat!(
                "        ", //
                "        ", //
                "        ", //
                "        ", //
                "   ##   ", //
                "  #  #  ", //
                "  #  #  ", //
                " #    # ", //
                " #    # ", //
                " ###### ", //
                " #    # ", //
                " #    # ", //
                " #    # ", //
                " #    # ", //
                "        ", //
                "        ", //
            )
        );
    }

    #[test]
    fn pixel_getter_outside() {
        let (chardata, _) = test_data();
        let glyph = parse_glyph(chardata);

        assert_eq!(glyph.pixel(8, 0), None);
        assert_eq!(glyph.pixel(0, 16), None);
        assert_eq!(glyph.pixel(8, 16), None);
    }

    #[test]
    fn parse_glyph_with_no_encoding() {
        let chardata = indoc! {r#"
            STARTCHAR 000
            ENCODING -1
            SWIDTH 432 0
            DWIDTH 6 0
            BBX 0 0 0 0
            BITMAP
            ENDCHAR
        "#};

        assert_eq!(
            parse_glyph(chardata),
            Glyph {
                bitmap: vec![],
                bounding_box: BoundingBox {
                    size: Coord::new(0, 0),
                    offset: Coord::new(0, 0),
                },
                encoding: Encoding::Unspecified,
                name: "000".to_string(),
                scalable_width: Some(Coord::new(432, 0)),
                device_width: Coord::new(6, 0),
            }
        );
    }

    #[test]
    fn parse_glyph_with_no_encoding_and_index() {
        let chardata = indoc! {r#"
            STARTCHAR 000
            ENCODING -1 123
            SWIDTH 432 0
            DWIDTH 6 0
            BBX 0 0 0 0
            BITMAP
            ENDCHAR
        "#};

        assert_eq!(
            parse_glyph(chardata),
            Glyph {
                bitmap: vec![],
                bounding_box: BoundingBox {
                    size: Coord::new(0, 0),
                    offset: Coord::new(0, 0),
                },
                encoding: Encoding::NonStandard(123),
                name: "000".to_string(),
                scalable_width: Some(Coord::new(432, 0)),
                device_width: Coord::new(6, 0),
            }
        );
    }

    #[test]
    fn parse_glyph_with_empty_bitmap() {
        let chardata = indoc! {r#"
            STARTCHAR 000
            ENCODING 0
            SWIDTH 432 0
            DWIDTH 6 0
            BBX 0 0 0 0
            BITMAP
            ENDCHAR
        "#};

        assert_eq!(
            parse_glyph(chardata),
            Glyph {
                bitmap: vec![],
                bounding_box: BoundingBox {
                    size: Coord::new(0, 0),
                    offset: Coord::new(0, 0),
                },
                encoding: Encoding::Standard(0),
                name: "000".to_string(),
                scalable_width: Some(Coord::new(432, 0)),
                device_width: Coord::new(6, 0),
            }
        );
    }
}

use nom::{
    bytes::complete::tag, character::complete::multispace0, character::complete::space1,
    combinator::map, combinator::opt, multi::many0, sequence::separated_pair, IResult,
};
use std::str::FromStr;

mod glyph;
mod helpers;
mod metadata;
mod properties;

pub use glyph::Glyph;
use helpers::*;
pub use metadata::Metadata;
pub use properties::{Properties, Property, PropertyValue};

/// BDF Font.
#[derive(Debug, Clone, PartialEq)]
pub struct BdfFont {
    /// Font metadata.
    pub metadata: Option<Metadata>,
    /// Glyphs.
    pub glyphs: Vec<Glyph>,
    /// Properties.
    pub properties: Properties,
}

impl BdfFont {
    fn parse(input: &str) -> IResult<&str, Self> {
        let (input, metadata) = opt(Metadata::parse)(input)?;
        let (input, _) = multispace0(input)?;
        let (input, properties) = Properties::parse(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = opt(numchars)(input)?;
        let (input, _) = multispace0(input)?;
        let (input, glyphs) = many0(Glyph::parse)(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = opt(tag("ENDFONT"))(input)?;
        let (input, _) = multispace0(input)?;

        Ok((
            input,
            Self {
                properties,
                metadata,
                glyphs,
            },
        ))
    }
}

impl FromStr for BdfFont {
    // TODO: use better error type
    type Err = ();

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let (remaining_input, font) = BdfFont::parse(source).map_err(|_| ())?;

        //TODO: can this happen?
        if !remaining_input.is_empty() {
            return Err(());
        }

        Ok(font)
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct BoundingBox {
    pub size: (u32, u32),
    pub offset: (i32, i32),
}

impl BoundingBox {
    pub(crate) fn parse(input: &str) -> IResult<&str, Self> {
        map(
            separated_pair(unsigned_xy, space1, signed_xy),
            |(size, offset)| Self { size, offset },
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_a_font_file() {
        let chardata = r#"STARTFONT 2.1
FONT "test font"
SIZE 16 75 75
FONTBOUNDINGBOX 16 24 0 0
STARTPROPERTIES 3
COPYRIGHT "https://github.com/iconic/open-iconic, SIL OPEN FONT LICENSE"
FONT_ASCENT 0
FONT_DESCENT 0
ENDPROPERTIES
STARTCHAR 000
ENCODING 64
DWIDTH 8 0
BBX 8 8 0 0
BITMAP
1f
01
ENDCHAR
STARTCHAR 000
ENCODING 64
DWIDTH 8 0
BBX 8 8 0 0
BITMAP
2f
02
ENDCHAR
ENDFONT
"#;

        let font = BdfFont::from_str(chardata).unwrap();

        assert_eq!(
            font.metadata,
            Some(Metadata {
                version: 2.1,
                name: String::from("\"test font\""),
                point_size: 16,
                resolution: (75, 75),
                bounding_box: BoundingBox {
                    size: (16, 24),
                    offset: (0, 0),
                },
            })
        );

        assert_eq!(
            font.glyphs,
            vec![
                Glyph {
                    bitmap: vec![0x1f, 0x01],
                    bounding_box: BoundingBox {
                        size: (8, 8),
                        offset: (0, 0),
                    },
                    encoding: Some('@'), //64
                    name: "000".to_string(),
                    device_width: Some((8, 0)),
                    scalable_width: None,
                },
                Glyph {
                    bitmap: vec![0x2f, 0x02],
                    bounding_box: BoundingBox {
                        size: (8, 8),
                        offset: (0, 0),
                    },
                    encoding: Some('@'), //64
                    name: "000".to_string(),
                    device_width: Some((8, 0)),
                    scalable_width: None,
                },
            ],
        );

        assert_eq!(
            font.properties.try_get(Property::Copyright),
            Ok("https://github.com/iconic/open-iconic, SIL OPEN FONT LICENSE".to_string())
        );
        assert_eq!(font.properties.try_get(Property::FontAscent), Ok(0));
        assert_eq!(font.properties.try_get(Property::FontDescent), Ok(0));
    }

    #[test]
    fn it_parses_optional_endfont_tag() {
        let chardata = r#"STARTFONT 2.1
FONT "open_iconic_all_1x"
SIZE 16 75 75
FONTBOUNDINGBOX 16 16 0 0
STARTPROPERTIES 3
COPYRIGHT "https://github.com/iconic/open-iconic, SIL OPEN FONT LICENSE"
FONT_ASCENT 0
FONT_DESCENT 0
ENDPROPERTIES
STARTCHAR 000
ENCODING 64
DWIDTH 8 0
BBX 8 8 0 0
BITMAP
1f
01
ENDCHAR
STARTCHAR 000
ENCODING 64
DWIDTH 8 0
BBX 8 8 0 0
BITMAP
2f
02
ENDCHAR
"#;

        let font = BdfFont::from_str(chardata).unwrap();

        assert_eq!(
            font.metadata,
            Some(Metadata {
                version: 2.1,
                name: String::from("\"open_iconic_all_1x\""),
                point_size: 16,
                resolution: (75, 75),
                bounding_box: BoundingBox {
                    size: (16, 16),
                    offset: (0, 0),
                },
            }),
        );

        assert_eq!(
            font.glyphs,
            vec![
                Glyph {
                    bitmap: vec![0x1f, 0x01],
                    bounding_box: BoundingBox {
                        size: (8, 8),
                        offset: (0, 0),
                    },
                    encoding: Some('@'), //64
                    name: "000".to_string(),
                    device_width: Some((8, 0)),
                    scalable_width: None,
                },
                Glyph {
                    bitmap: vec![0x2f, 0x02],
                    bounding_box: BoundingBox {
                        size: (8, 8),
                        offset: (0, 0),
                    },
                    encoding: Some('@'), //64
                    name: "000".to_string(),
                    device_width: Some((8, 0)),
                    scalable_width: None,
                }
            ]
        );

        assert_eq!(
            font.properties.try_get(Property::Copyright),
            Ok("https://github.com/iconic/open-iconic, SIL OPEN FONT LICENSE".to_string())
        );
        assert_eq!(font.properties.try_get(Property::FontAscent), Ok(0));
        assert_eq!(font.properties.try_get(Property::FontDescent), Ok(0));
    }

    #[test]
    fn it_handles_windows_line_endings() {
        let windows_line_endings = "STARTFONT 2.1\r\nFONT \"windows_test\"\r\nSIZE 10 96 96\r\nFONTBOUNDINGBOX 8 16 0 -4\r\nCHARS 256\r\nSTARTCHAR 0\r\nENCODING 0\r\nSWIDTH 600 0\r\nDWIDTH 8 0\r\nBBX 8 16 0 -4\r\nBITMAP\r\nD5\r\nENDCHAR\r\nENDFONT\r\n";

        let font = BdfFont::from_str(windows_line_endings).unwrap();

        assert_eq!(
            font.metadata,
            Some(Metadata {
                version: 2.1,
                name: String::from("\"windows_test\""),
                point_size: 10,
                resolution: (96, 96),
                bounding_box: BoundingBox {
                    size: (8, 16),
                    offset: (0, -4),
                },
            })
        );

        assert_eq!(
            font.glyphs,
            vec![Glyph {
                bitmap: vec![0xd5],
                bounding_box: BoundingBox {
                    size: (8, 16),
                    offset: (0, -4),
                },
                encoding: Some('\x00'),
                name: "0".to_string(),
                device_width: Some((8, 0)),
                scalable_width: Some((600, 0)),
            },]
        );

        assert!(font.properties.is_empty());
    }
}

use std::str::FromStr;

use nom::{
    bytes::complete::tag,
    character::complete::multispace0,
    character::complete::space1,
    combinator::map,
    combinator::{eof, opt},
    multi::many0,
    sequence::{separated_pair, terminated},
    IResult,
};

mod glyph;
mod helpers;
mod metadata;
mod properties;

use glyph::*;
use helpers::*;
use metadata::*;
use properties::*;

#[derive(Debug, Clone, PartialEq)]
pub struct BdfFont {
    metadata: Option<Metadata>,
    glyphs: Vec<Glyph>,
    properties: Option<Properties>,
}

impl BdfFont {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, metadata) = opt(header)(input)?;
        let (input, _) = multispace0(input)?;
        let (input, properties) = opt(properties)(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = opt(numchars)(input)?;
        let (input, _) = multispace0(input)?;
        let (input, glyphs) = many0(glyph)(input)?;
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

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let (_, font) = terminated(Self::parse, eof)(&input.as_bytes()).map_err(|_| ())?;

        Ok(font)
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct BoundingBox {
    pub size: (u32, u32),
    pub offset: (i32, i32),
}

impl BoundingBox {
    pub(crate) fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        map(
            separated_pair(unsigned_xy, space1, signed_xy),
            |(size, offset)| Self { size, offset },
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashmap;

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
            font,
            BdfFont {
                metadata: Some(Metadata {
                    version: 2.1,
                    name: String::from("\"test font\""),
                    size: (16, (75, 75)),
                    bounding_box: BoundingBox {
                        size: (16, 24),
                        offset: (0, 0),
                    }
                }),
                glyphs: vec![
                    Glyph {
                        bitmap: vec![0x1f01],
                        bounding_box: BoundingBox {
                            size: (8, 8),
                            offset: (0, 0),
                        },
                        charcode: 64,
                        name: "000".to_string(),
                    },
                    Glyph {
                        bitmap: vec![0x2f02],
                        bounding_box: BoundingBox {
                            size: (8, 8),
                            offset: (0, 0),
                        },
                        charcode: 64,
                        name: "000".to_string(),
                    },
                ],
                properties: Some(hashmap! {
                    "COPYRIGHT".into() => PropertyValue::Text("https://github.com/iconic/open-iconic, SIL OPEN FONT LICENSE".into()),
                    "FONT_ASCENT".into() => PropertyValue::Int(0),
                    "FONT_DESCENT".into() => PropertyValue::Int(0),
                })
            }
        );
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
            font,
            BdfFont {
                metadata: Some(Metadata {
                    version: 2.1,
                    name: String::from("\"open_iconic_all_1x\""),
                    size: (16, (75, 75)),
                    bounding_box: BoundingBox {
                        size: (16, 16),
                        offset: (0, 0),
                    }
                }),
                glyphs: vec![
                    Glyph {
                        bitmap: vec![0x1f01],
                        bounding_box: BoundingBox {
                            size: (8, 8),
                            offset: (0, 0)
                        },
                        charcode: 64,
                        name: "000".to_string(),
                    },
                    Glyph {
                        bitmap: vec![0x2f02],
                        bounding_box: BoundingBox {
                            size: (8, 8),
                            offset: (0, 0)
                        },
                        charcode: 64,
                        name: "000".to_string(),
                    },
                ],
                properties: Some(hashmap! {
                    "COPYRIGHT".into() => PropertyValue::Text("https://github.com/iconic/open-iconic, SIL OPEN FONT LICENSE".into()),
                    "FONT_ASCENT".into() => PropertyValue::Int(0),
                    "FONT_DESCENT".into() => PropertyValue::Int(0),
                })
            }
        );
    }

    #[test]
    fn it_handles_windows_line_endings() {
        let windows_line_endings = "STARTFONT 2.1\r\nFONT \"windows_test\"\r\nSIZE 10 96 96\r\nFONTBOUNDINGBOX 8 16 0 -4\r\nCHARS 256\r\nSTARTCHAR 0\r\nENCODING 0\r\nSWIDTH 600 0\r\nDWIDTH 8 0\r\nBBX 8 16 0 -4\r\nBITMAP\r\nD5\r\nENDCHAR\r\nENDFONT\r\n";

        let font = BdfFont::from_str(windows_line_endings).unwrap();

        assert_eq!(
            font,
            BdfFont {
                metadata: Some(Metadata {
                    version: 2.1,
                    name: String::from("\"windows_test\""),
                    size: (10, (96, 96)),
                    bounding_box: BoundingBox {
                        size: (8, 16),
                        offset: (0, -4)
                    },
                }),
                glyphs: vec![Glyph {
                    bitmap: vec![0xd5],
                    bounding_box: BoundingBox {
                        size: (8, 16),
                        offset: (0, -4)
                    },
                    charcode: 0,
                    name: "0".to_string(),
                },],
                properties: None
            }
        );
    }
}

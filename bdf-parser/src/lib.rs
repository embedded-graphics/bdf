#[macro_use]
extern crate nom;

mod glyph;
mod helpers;
mod metadata;
mod properties;

use glyph::*;
use helpers::*;
use metadata::*;
use properties::*;

pub type BoundingBox = (u32, u32, i32, i32);

#[derive(Debug, Clone, PartialEq)]
pub struct BDFFont {
    metadata: Option<Metadata>,
    glyphs: Vec<Glyph>,
}

pub struct BDFParser<'a> {
    source: &'a str,
}

impl<'a> BDFParser<'a> {
    pub fn from_str(source: &'a str) -> Self {
        Self { source }
    }

    pub fn parse(&self) -> nom::IResult<&[u8], BDFFont> {
        bdf(self.source.as_bytes())
    }
}

named!(
    inner_bdf<BDFFont>,
    ws!(do_parse!(
        metadata: opt!(header) >> opt!(properties) >> opt!(numchars) >> glyphs: many0!(glyph) >> ({
            BDFFont { metadata, glyphs }
        })
    ))
);

named!(
    bdf<BDFFont>,
    alt_complete!(ws!(terminated!(inner_bdf, tag!("ENDFONT"))) | inner_bdf)
);

#[cfg(test)]
mod tests {
    use super::*;
    use nom::IResult;

    const EMPTY: &[u8] = &[];

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

        let out = bdf(&chardata.as_bytes());

        assert_eq!(
            out,
            IResult::Done(
                EMPTY,
                BDFFont {
                    metadata: Some(Metadata {
                        version: 2.1,
                        name: <String>::from("\"test font\""),
                        size: (16, 75, 75),
                        bounding_box: (16, 24, 0, 0),
                    }),
                    glyphs: vec![
                        Glyph {
                            bitmap: vec![0x1f01],
                            bounding_box: (8, 8, 0, 0),
                            charcode: 64,
                            name: "000".to_string(),
                        },
                        Glyph {
                            bitmap: vec![0x2f02],
                            bounding_box: (8, 8, 0, 0),
                            charcode: 64,
                            name: "000".to_string(),
                        },
                    ],
                }
            )
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

        let out = bdf(&chardata.as_bytes());

        assert_eq!(
            out,
            IResult::Done(
                EMPTY,
                BDFFont {
                    metadata: Some(Metadata {
                        version: 2.1,
                        name: <String>::from("\"open_iconic_all_1x\""),
                        size: (16, 75, 75),
                        bounding_box: (16, 16, 0, 0),
                    }),
                    glyphs: vec![
                        Glyph {
                            bitmap: vec![0x1f01],
                            bounding_box: (8, 8, 0, 0),
                            charcode: 64,
                            name: "000".to_string(),
                        },
                        Glyph {
                            bitmap: vec![0x2f02],
                            bounding_box: (8, 8, 0, 0),
                            charcode: 64,
                            name: "000".to_string(),
                        },
                    ],
                }
            )
        );
    }
}

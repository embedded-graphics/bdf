#[macro_use]
extern crate nom;

use nom::*;

pub type FontSize = (u32, u32, u32);
pub type BoundingBox = (u32, u32, i32, i32);
type Vec2 = (u32, u32);

#[derive(Debug, Clone)]
pub struct Glyph {
    name: String,
    charcode: u32,
    bounding_box: BoundingBox,
    bitmap: Vec<u32>,
}

#[derive(Debug, Clone)]
struct Properties;

#[derive(Debug, Clone)]
struct Metadata {
    version: f32,
    name: String,
    size: FontSize,
    bounding_box: BoundingBox,
}

#[derive(Debug, Clone)]
pub struct BDFFont {
    metadata: Metadata,
    glyphs: Vec<Glyph>,
}

pub struct BDFParser<'a> {
    source: &'a str,
}

impl<'a> BDFParser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source }
    }

    pub fn parse(&self) -> Result<(&[u8], BDFFont), nom::Err<&[u8]>> {
        bdf(self.source.as_bytes())
    }
}

named!(
    parse_to_i32<i32>,
    flat_map!(
        recognize!(preceded!(opt!(one_of!("+-")), digit)),
        parse_to!(i32)
    )
);

named!(parse_to_u32<u32>, flat_map!(digit, parse_to!(u32)));

named!(startfont<f32>, ws!(preceded!(tag!("STARTFONT"), float)));

named!(
    fontname<String>,
    flat_map!(
        ws!(preceded!(tag!("FONT"), take_until!("\n"))),
        parse_to!(String)
    )
);

named!(
    size<FontSize>,
    ws!(preceded!(
        tag!("SIZE"),
        tuple!(parse_to_u32, parse_to_u32, parse_to_u32)
    ))
);

named!(
    boundingbox<BoundingBox>,
    ws!(tuple!(
        parse_to_u32,
        parse_to_u32,
        parse_to_i32,
        parse_to_i32
    ))
);

named!(
    fontboundingbox<BoundingBox>,
    ws!(preceded!(tag!("FONTBOUNDINGBOX"), boundingbox))
);

named!(
    startproperties<u32>,
    ws!(preceded!(tag!("STARTPROPERTIES"), parse_to_u32))
);

named!(
    fontascent<u32>,
    ws!(preceded!(tag!("FONT_ASCENT"), parse_to_u32))
);

named!(
    fontdescent<u32>,
    ws!(preceded!(tag!("FONT_DESCENT"), parse_to_u32))
);

named!(numchars<u32>, ws!(preceded!(tag!("CHARS"), parse_to_u32)));

named!(
    startchar<String>,
    flat_map!(
        ws!(preceded!(tag!("STARTCHAR"), take_until!("\n"))),
        parse_to!(String)
    )
);

named!(
    charcode<u32>,
    ws!(preceded!(tag!("ENCODING"), parse_to_u32))
);

named!(
    swidth<Vec2>,
    ws!(preceded!(
        tag!("SWIDTH"),
        tuple!(parse_to_u32, parse_to_u32)
    ))
);

named!(
    dwidth<Vec2>,
    ws!(preceded!(
        tag!("DWIDTH"),
        tuple!(parse_to_u32, parse_to_u32)
    ))
);

named!(
    charboundingbox<BoundingBox>,
    ws!(preceded!(tag!("BBX"), boundingbox))
);

named!(bitmap, terminated!(tag!("BITMAP"), line_ending));
named!(
    endproperties,
    terminated!(tag!("ENDPROPERTIES"), line_ending)
);
named!(endchar, terminated!(tag!("ENDCHAR"), line_ending));
named!(endfont, terminated!(tag!("ENDFONT"), line_ending));

named!(chardata<u32>, terminated!(hex_u32, line_ending));

named!(
    metadata<Metadata>,
    ws!(do_parse!(
        version: startfont >> name: fontname >> size: size >> bounding_box: fontboundingbox >> ({
            Metadata {
                version,
                name,
                size,
                bounding_box,
            }
        })
    ))
);

// TODO: Properties
named!(
    properties<Properties>,
    map!(
        ws!(preceded!(
            startproperties,
            take_until_and_consume!("ENDPROPERTIES")
        )),
        |_| Properties
    )
);

named!(
    glyph<Glyph>,
    do_parse!(
        name: startchar >> charcode: charcode >> swidth >> dwidth >> bounding_box: charboundingbox
            >> bitmap >> bitmap: ws!(many1!(chardata)) >> endchar >> ({
            Glyph {
                name,
                charcode,
                bounding_box,
                bitmap: bitmap,
            }
        })
    )
);

named!(
    bdf<BDFFont>,
    terminated!(
        do_parse!(
            metadata: metadata >> properties >> opt!(numchars) >> glyphs: many1!(glyph) >> ({
                BDFFont {
                    metadata,
                    glyphs: glyphs,
                }
            })
        ),
        endfont
    )
);

pub fn parse_char(input: &str) -> Result<(&[u8], Glyph), nom::Err<&[u8]>> {
    glyph(input.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_a_single_char() {
        let chardata = r#"STARTCHAR U+0041
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
"#;

        let out = parse_char(&chardata);

        assert!(out.is_ok());

        assert_eq!(out.unwrap().0.len(), 0);
    }
}

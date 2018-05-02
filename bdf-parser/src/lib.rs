#[macro_use]
extern crate nom;

use nom::*;

type FontSize = (u32, u32, u32);
type BoundingBox = (u32, u32, i32, i32);

#[derive(Debug, Clone)]
struct Glyph {
    charcode: u32,
    bounding_box: BoundingBox,
    bitmap: Vec<u8>,
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

enum Token {
    Byte(u8),
    CharBoundingBox(BoundingBox),
    NumChars(u32),
    DeviceWidth((u32, u32)),
    Encoding(u32),
    EndChar,
    EndFont,
    EndProperties,
    Font(String),
    FontAscent(u32),
    FontBoundingBox(BoundingBox),
    FontDescent(u32),
    ScalableWidth((u32, u32)),
    Size(FontSize),
    StartBitmap,
    StartChar,
    StartFont(f32),
    StartProperties,
}

// Parser helpers
// named!(
//     parse_to_i32<i32>,
//     flat_map!(
//         recognize!(alt!(preceded!(tag!("-"), digit) | digit)),
//         parse_to!(i32)
//     )
// );

named!(
    parse_to_i32<i32>,
    flat_map!(
        recognize!(preceded!(opt!(one_of!("+-")), digit)),
        parse_to!(i32)
    )
);

// Token parsers
named!(startfont<f32>, ws!(preceded!(tag!("STARTFONT"), float)));

named!(
    font<String>,
    flat_map!(
        ws!(preceded!(tag!("FONT"), take_until!("\n"))),
        parse_to!(String)
    )
);

named!(
    size<FontSize>,
    ws!(preceded!(
        tag!("SIZE"),
        tuple!(
            flat_map!(digit, parse_to!(u32)),
            flat_map!(digit, parse_to!(u32)),
            flat_map!(digit, parse_to!(u32))
        )
    ))
);

named!(
    fontboundingbox<BoundingBox>,
    ws!(preceded!(
        tag!("FONTBOUNDINGBOX"),
        tuple!(
            flat_map!(digit, parse_to!(u32)),
            flat_map!(digit, parse_to!(u32)),
            parse_to_i32,
            parse_to_i32
        )
    ))
);

named!(
    startproperties<Token>,
    map!(tag!("STARTPROPERTIES"), |_| Token::StartProperties)
);

named!(
    fontascent<Token>,
    map!(
        ws!(preceded!(
            tag!("FONT_ASCENT"),
            flat_map!(digit, parse_to!(u32))
        )),
        Token::FontAscent
    )
);

named!(
    fontdescent<Token>,
    map!(
        ws!(preceded!(
            tag!("FONT_DESCENT"),
            flat_map!(digit, parse_to!(u32))
        )),
        Token::FontDescent
    )
);

named!(
    endproperties<Token>,
    map!(tag!("ENDPROPERTIES"), |_| Token::EndProperties)
);

named!(
    numchars<Token>,
    map!(
        ws!(preceded!(tag!("CHARS"), flat_map!(digit, parse_to!(u32)))),
        Token::NumChars
    )
);

named!(
    startchar<Token>,
    map!(ws!(preceded!(tag!("STARTCHAR"), take_until!("\n"))), |_| {
        Token::StartChar
    })
);

named!(
    charcode<u32>,
    ws!(preceded!(
        tag!("ENCODING"),
        flat_map!(digit, parse_to!(u32))
    ))
);

named!(
    swidth<Token>,
    map!(
        ws!(preceded!(
            tag!("SWIDTH"),
            tuple!(
                flat_map!(digit, parse_to!(u32)),
                flat_map!(digit, parse_to!(u32))
            )
        )),
        Token::ScalableWidth
    )
);

named!(
    dwidth<Token>,
    map!(
        ws!(preceded!(
            tag!("DWIDTH"),
            tuple!(
                flat_map!(digit, parse_to!(u32)),
                flat_map!(digit, parse_to!(u32))
            )
        )),
        Token::DeviceWidth
    )
);

named!(
    bbx<BoundingBox>,
    ws!(preceded!(
        tag!("BBX"),
        tuple!(
            flat_map!(digit, parse_to!(u32)),
            flat_map!(digit, parse_to!(u32)),
            parse_to_i32,
            parse_to_i32
        )
    ))
);

named!(bitmap<Token>, map!(tag!("BITMAP"), |_| Token::StartBitmap));

named!(chardata<u8>, map!(ws!(hex_u32), |res| res as u8));

named!(
    endchar<Token>,
    map!(ws!(tag!("ENDCHAR")), |_| Token::EndChar)
);

named!(
    endfont<Token>,
    map!(ws!(tag!("ENDFONT")), |_| Token::EndFont)
);

named!(
    metadata<Metadata>,
    ws!(do_parse!(
        version: startfont >> name: font >> size: size >> bounding_box: fontboundingbox >> ({
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
    ws!(do_parse!(
        startchar >> charcode: charcode >> swidth >> dwidth >> bounding_box: bbx >> bitmap
            >> bitmap: many_till!(chardata, endchar) >> ({
            Glyph {
                charcode,
                bounding_box,
                bitmap: bitmap.0,
            }
        })
    ))
);

named!(
    bdf<BDFFont>,
    ws!(do_parse!(
        metadata: metadata >> properties >> opt!(numchars) >> glyphs: many1!(glyph) >> ({
            BDFFont {
                metadata,
                glyphs: glyphs,
            }
        })
    ))
);

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

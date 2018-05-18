use nom::types::CompleteByteSlice;

use super::BoundingBox;
use super::helpers::*;

pub type FontSize = (i32, u32, u32);

#[derive(Debug, Clone, PartialEq)]
pub struct Metadata {
    pub version: f32,
    pub name: String,
    pub size: FontSize,
    pub bounding_box: BoundingBox,
}

named!(
    metadata_version<CompleteByteSlice, f32>,
    flat_map!(
        ws!(preceded!(tag!("STARTFONT"), take_until_line_ending)),
        parse_to!(f32)
    )
);

named!(
    metadata_name<CompleteByteSlice, String>,
    flat_map!(
        preceded!(tag!("FONT "), take_until_line_ending),
        parse_to!(String)
    )
);

named!(
    metadata_size<CompleteByteSlice, FontSize>,
    ws!(preceded!(
        tag!("SIZE"),
        tuple!(parse_to_i32, parse_to_u32, parse_to_u32)
    ))
);

named!(
    metadata_bounding_box<CompleteByteSlice, BoundingBox>,
    ws!(preceded!(
        tag!("FONTBOUNDINGBOX"),
        tuple!(parse_to_u32, parse_to_u32, parse_to_i32, parse_to_i32)
    ))
);

named!(
    pub header<CompleteByteSlice, Metadata>,
    ws!(do_parse!(
        optional_comments >> version: metadata_version >> optional_comments >> name: metadata_name
            >> optional_comments >> size: metadata_size >> optional_comments
            >> bounding_box: metadata_bounding_box >> optional_comments >> ({
            Metadata {
                version,
                name,
                size,
                bounding_box,
            }
        })
    ))
);

#[cfg(test)]
mod tests {
    use super::*;

    const EMPTY: CompleteByteSlice = CompleteByteSlice(b"");

    #[test]
    fn it_parses_the_font_version() {
        assert_eq!(
            metadata_version(CompleteByteSlice(b"STARTFONT 2.1\n")),
            Ok((EMPTY, 2.1f32))
        );

        // Some fonts are a bit overzealous with their whitespace
        assert_eq!(
            metadata_version(CompleteByteSlice(b"STARTFONT   2.1\n")),
            Ok((EMPTY, 2.1f32))
        );
    }
}

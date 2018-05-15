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
    metadata_version<f32>,
    flat_map!(
        preceded!(tag!("STARTFONT "), take_until_line_ending),
        parse_to!(f32)
    )
);

named!(
    metadata_name<String>,
    flat_map!(
        preceded!(tag!("FONT "), take_until_line_ending),
        parse_to!(String)
    )
);

named!(
    metadata_size<FontSize>,
    ws!(preceded!(
        tag!("SIZE"),
        tuple!(parse_to_i32, parse_to_u32, parse_to_u32)
    ))
);

named!(
    metadata_bounding_box<BoundingBox>,
    ws!(preceded!(
        tag!("FONTBOUNDINGBOX"),
        tuple!(parse_to_u32, parse_to_u32, parse_to_i32, parse_to_i32)
    ))
);

named!(
    pub header<Metadata>,
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

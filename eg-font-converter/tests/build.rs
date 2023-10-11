use eg_font_converter::*;
use embedded_graphics::{
    geometry::AnchorX,
    image::{GetPixel, ImageRaw},
    pixelcolor::BinaryColor,
    prelude::*,
};
use pretty_assertions::assert_eq;

fn data_to_string(data: &[u8], width: u32) -> String {
    let image = ImageRaw::<BinaryColor>::new(data, width);
    let bb = image.bounding_box();

    let mut str = String::new();
    for p in bb.points() {
        if image.pixel(p).unwrap().is_on() {
            str.push('#');
        } else {
            str.push('.');
        }
        if p.x == bb.anchor_x(AnchorX::Right) {
            str.push('\n');
        }
    }

    str
}

fn assert_data(data: &[u8], expected: &[u8], width: u32) {
    assert_eq!(data.len(), expected.len());

    let data_str = data_to_string(data, width);
    let expected_str = data_to_string(expected, width);
    assert_eq!(data_str, expected_str);
}

#[test]
fn eg_bdf_az() {
    let font = FontConverter::new("../eg-bdf-examples/examples/6x10.bdf", "EG_BDF_AZ")
        .glyphs('a'..='z')
        .convert_eg_bdf()
        .unwrap();

    assert_eq!(font.rust(), include_str!("expected/eg_bdf_az.rs"));
}

#[test]
fn mono_font_6x10_ascii() {
    let font_6x10 = FontConverter::new(
        "../eg-bdf-examples/examples/6x10.bdf",
        "MONO_FONT_6X10_ASCII",
    )
    .glyphs(Mapping::Ascii)
    .missing_glyph_substitute('?')
    .convert_mono_font()
    .expect("couldn't convert font");

    assert_eq!(
        font_6x10.rust(),
        include_str!("expected/mono_font_6x10_ascii.rs")
    );
    assert_data(
        font_6x10.data(),
        include_bytes!("expected/mono_font_6x10_ascii.data"),
        6 * 16,
    );
}

#[test]
fn mono_font_10x20_iso8859_1() {
    let font_10x20 = FontConverter::new(
        "../eg-bdf-examples/examples/10x20.bdf",
        "MONO_FONT_10X20_ISO8859_1",
    )
    .glyphs(Mapping::Iso8859_1)
    .missing_glyph_substitute('?')
    .convert_mono_font()
    .expect("couldn't convert font");

    assert_eq!(
        font_10x20.rust(),
        include_str!("expected/mono_font_10x20_iso8859_1.rs")
    );
    assert_data(
        font_10x20.data(),
        include_bytes!("expected/mono_font_10x20_iso8859_1.data"),
        10 * 16,
    );
}

#[test]
fn mono_font_10x20_iso8859_15() {
    let font_10x20 = FontConverter::new(
        "../eg-bdf-examples/examples/10x20.bdf",
        "MONO_FONT_10X20_ISO8859_15",
    )
    .glyphs(Mapping::Iso8859_15)
    .missing_glyph_substitute('?')
    .convert_mono_font()
    .expect("couldn't convert font");

    assert_eq!(
        font_10x20.rust(),
        include_str!("expected/mono_font_10x20_iso8859_15.rs")
    );
    assert_data(
        font_10x20.data(),
        include_bytes!("expected/mono_font_10x20_iso8859_15.data"),
        10 * 16,
    );
}

#[test]
fn mono_font_6x10_az() {
    let font_6x10 = FontConverter::new("../eg-bdf-examples/examples/6x10.bdf", "FONT_6X10_AZ")
        .glyphs('a'..='z')
        .comment("6x10 pixel monospace font.")
        .convert_mono_font()
        .unwrap();

    assert_eq!(
        font_6x10.rust(),
        include_str!("expected/mono_font_6x10_az.rs")
    );
    assert_data(
        font_6x10.data(),
        include_bytes!("expected/mono_font_6x10_az.data"),
        6 * 16,
    );
}

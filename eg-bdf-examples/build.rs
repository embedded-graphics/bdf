use eg_font_converter::{FontConverter, Mapping};

fn main() {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();

    // Convert to eg-bdf fonts.

    let font_6x10 = FontConverter::with_file("examples/6x10.bdf", "FONT_6X10")
        //.file_stem("font_6x10")
        .glyphs(Mapping::Iso8859_15)
        .missing_glyph_substitute(' ')
        .convert_eg_bdf()
        .unwrap();

    let font_10x20 = FontConverter::with_file("examples/10x20.bdf", "FONT_10X20")
        //.file_stem("font_10x20")
        .glyphs(Mapping::Iso8859_15)
        //.glyphs('A'..='Z')
        .missing_glyph_substitute(' ')
        .convert_eg_bdf()
        .unwrap();

    font_6x10.save(&out_dir).unwrap();
    font_10x20.save(&out_dir).unwrap();

    // // Convert to MonoFont fonts.

    let font_6x10 = FontConverter::with_file("examples/6x10.bdf", "FONT_6X10_MONO")
        // .file_stem("font_6x10_mono")
        .glyphs(Mapping::Iso8859_15)
        .missing_glyph_substitute(' ')
        .convert_mono_font()
        .unwrap();

    let font_10x20 = FontConverter::with_file("examples/10x20.bdf", "FONT_10X20_MONO")
        // .name("FONT_10X20_MONO")
        // .file_stem("font_10x20_mono")
        .glyphs(Mapping::Iso8859_15)
        .missing_glyph_substitute(' ')
        .convert_mono_font()
        .unwrap();

    font_6x10.save(&out_dir).unwrap();
    font_10x20.save(&out_dir).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    //TODO: add font files
}

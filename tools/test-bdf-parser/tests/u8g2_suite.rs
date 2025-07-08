use std::path::Path;

use test_bdf_parser::*;

fn filter(path: &Path) -> bool {
    match path.file_name().unwrap().to_str().unwrap() {
        "u8x8extra.bdf" => false,   // broken header
        "emoticons21.bdf" => false, // invalid COPYRIGHT property in line 7
        _ => true,
    }
}

#[test]
fn it_parses_all_u8g2_fonts() {
    let fontdir = Path::new("../../target/fonts/u8g2/tools/font/bdf")
        .canonicalize()
        .unwrap();

    let fonts = parse_fonts_with_filter(&fontdir, filter).expect("Could not parse fonts");
    let num_errors = print_parser_result(&fonts);

    assert_eq!(num_errors, 0, "Not all font files parsed successfully");
}

use std::path::Path;

use test_bdf_parser::*;

#[test]
fn it_parses_all_tecate_fonts() {
    let fontdir = Path::new("../../target/fonts/bitmap-fonts/bitmap")
        .canonicalize()
        .unwrap();

    let fonts = parse_fonts(&fontdir).expect("Could not parse fonts");
    let num_errors = print_parser_result(&fonts);

    assert_eq!(num_errors, 0, "Not all font files parsed successfully");
}

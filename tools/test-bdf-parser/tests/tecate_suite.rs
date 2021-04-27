use std::path::Path;

use test_bdf_parser::*;

#[test]
fn it_parses_all_tecate_fonts() {
    let fontdir = Path::new("../../target/fonts/bitmap-fonts/bitmap")
        .canonicalize()
        .unwrap();

    let fonts = collect_font_files(&fontdir).expect("Could not get list of fonts");

    let results = fonts.iter().map(|fpath| test_font_parse(fpath));

    let mut num_errors = 0;

    for (font, result) in fonts.iter().zip(results) {
        if result.is_err() {
            num_errors += 1;
        }

        println!(
            "{0: <60} {1:?}",
            font.file_name().unwrap().to_str().unwrap(),
            result
        );
    }

    println!(
        "\n{} out of {} fonts passed ({} failed)\n",
        (fonts.len() - num_errors),
        fonts.len(),
        num_errors
    );

    assert_eq!(num_errors, 0, "Not all font files parsed successfully");
}

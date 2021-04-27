use clap::Clap;
use std::path::PathBuf;

use test_bdf_parser::*;

#[derive(Clap)]
struct Arguments {
    file_or_directory: PathBuf,
}

pub fn main() {
    let args: Arguments = Arguments::parse();

    if args.file_or_directory.is_dir() {
        let fonts =
            collect_font_files(&args.file_or_directory).expect("Could not get list of fonts");

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
    } else if args.file_or_directory.is_file() {
        test_font_parse(&args.file_or_directory).unwrap();
    } else {
        panic!("Invalid path: {:?}", args.file_or_directory);
    }
}

extern crate bdf_parser;
extern crate chardet;
extern crate encoding;
extern crate nom;

use chardet::{charset2encoding, detect};
use encoding::label::encoding_from_whatwg_label;
use encoding::DecoderTrap;
use std::fs::File;
use std::fs::OpenOptions;
use std::fs::{self, DirEntry};
use std::io;
use std::io::prelude::*;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use bdf_parser::*;
use nom::*;

// // one possible implementation of walking a directory only visiting files
// fn visit_dirs(dir: &Path, cb: &Fn(&DirEntry)) -> io::Result<()> {
//     if dir.is_dir() {
//         for entry in fs::read_dir(dir)? {
//             let entry = entry?;
//             let path = entry.path();
//             if path.is_dir() {
//                 visit_dirs(&path, cb)?;
//             } else {
//                 cb(&entry);
//             }
//         }
//     }
//     Ok(())
// }

fn collect_font_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                files.push(path.to_path_buf());
            }
        }
    }

    Ok(files)
}

fn read(path: &Path) -> String {
    // let mut file = File::open(path).expect("Unable to open file");
    // let mut contents = String::new();
    // file.read_to_string(&mut contents)
    //     .expect(&format!("Unable to read file {:?}", path));

    // open text file
    let mut fh = OpenOptions::new()
        .read(true)
        .open(path)
        .expect("Could not open file");
    let mut reader: Vec<u8> = Vec::new();

    // read file
    fh.read_to_end(&mut reader).expect("Could not read file");

    // detect charset of the file
    let result = detect(&reader);
    // result.0 Encode
    // result.1 Confidence
    // result.2 Language

    // decode file into utf-8
    let coder = encoding_from_whatwg_label(charset2encoding(&result.0));
    // if coder.is_some() {
    let utf8reader = coder
        .unwrap()
        .decode(&reader, DecoderTrap::Ignore)
        .expect("Error");
    // }

    // contents

    utf8reader
}

fn test_font_parse(filepath: &Path) -> Result<(), String> {
    let bdf = read(filepath);

    let parser = BDFParser::new(&bdf);

    let out = parser.parse();

    match out {
        Ok((rest, parsed)) => {
            // println!("Rest: {:?}", rest);

            if rest.len() > 0 {
                Err(format!("{} remaining bytes to parse", rest.len()))
            } else {
                Ok(())
            }
        }
        Err(err) => match err {
            nom::Err::Incomplete(need) => Err(format!("Incomplete, need {:?} more", need)),
            nom::Err::Error(Context::Code(c, error_kind)) => {
                #[cfg(debug_assertions)]
                let name = filepath.file_name().unwrap().to_str().unwrap();
                #[cfg(debug_assertions)]
                println!(
                    "{} Error\n    {:?}",
                    name,
                    String::from_utf8(c.to_vec()).unwrap()
                );
                #[cfg(debug_assertions)]
                panic!();

                Err(format!("Parse error {:?}", error_kind))
            }
            nom::Err::Failure(_) => Err(format!("Unrecoverable parse error")),
            nom::Err::Error(l) => panic!("Idk {:?}", l),
        },
    }
}

#[test]
fn it_parses_all_u8g2_fonts() {
    let fontdir = Path::new("./tests/u8g2/tools/font/bdf")
        .canonicalize()
        .unwrap();

    // println!("Font dir {:?}", fontdir);

    let fonts = collect_font_files(&fontdir).expect("Could not get list of u8g2 fonts");

    // println!("{:?}", fonts);

    let results = fonts.iter().map(|fpath| test_font_parse(fpath));

    let mut num_errors = 0;

    for (font, result) in fonts.iter().zip(results) {
        if result.is_err() {
            num_errors += 1;
        }

        println!(
            "{0: <30} {1:?}",
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

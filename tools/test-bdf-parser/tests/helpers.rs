use chardet::{charset2encoding, detect};
use encoding::{label::encoding_from_whatwg_label, DecoderTrap};
use std::{
    fs,
    fs::OpenOptions,
    io,
    io::prelude::*,
    path::{Path, PathBuf},
};

use bdf_parser::*;

pub fn collect_font_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.to_string_lossy().ends_with(".bdf") {
                files.push(path.to_path_buf());
            } else if path.is_dir() {
                let sub = collect_font_files(&path).unwrap();
                for subfile in sub {
                    files.push(subfile);
                }
            }
        }
    }

    files.sort();

    Ok(files)
}

pub fn read(path: &Path) -> String {
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

    let utf8reader = coder
        .unwrap()
        .decode(&reader, DecoderTrap::Ignore)
        .expect("Error");

    utf8reader
}

pub fn test_font_parse(filepath: &Path) -> Result<(), String> {
    let bdf = read(filepath);

    let font = bdf.parse::<BdfFont>();

    match font {
        Ok(_font) => Ok(()),
        _ => Err(format!("Other error")),
    }
}

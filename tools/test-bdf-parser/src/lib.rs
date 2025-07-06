use std::{
    fs, io,
    path::{Path, PathBuf},
};

use bdf_parser::Font;

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

pub fn test_font_parse(filepath: &Path) -> Result<(), String> {
    let bdf = std::fs::read(filepath).unwrap();
    let str = String::from_utf8_lossy(&bdf);
    let font = Font::parse(&str);

    match font {
        Ok(_font) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

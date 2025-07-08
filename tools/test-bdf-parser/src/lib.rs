use anyhow::Result;
use owo_colors::OwoColorize;
use std::{
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
};

use bdf_parser::{Font, ParserError};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FontPath {
    pub absolute: PathBuf,
    pub relative: PathBuf,
}

#[derive(Debug)]
pub struct FontFile {
    pub path: FontPath,
    pub parsed: Result<Font, ParserError>,
}

#[derive(Debug, Default)]
struct DirWalker {
    files: Vec<FontPath>,

    prefix: PathBuf,
}

impl DirWalker {
    fn new<F: Fn(&Path) -> bool>(path: &Path, filter: F) -> io::Result<Self> {
        let mut self_ = Self::default();

        self_.walk(path, &filter, true)?;
        self_.files.sort();

        Ok(self_)
    }

    fn walk<F: Fn(&Path) -> bool>(
        &mut self,
        path: &Path,
        filter: &F,
        root: bool,
    ) -> io::Result<()> {
        let file_name = path.file_name().unwrap();

        if path.is_dir() {
            let old_prefix = self.prefix.clone();
            if !root {
                self.prefix.push(file_name);
            }

            for entry in fs::read_dir(path)? {
                let entry = entry?;

                self.walk(&entry.path(), filter, false)?;
            }

            self.prefix = old_prefix;
        } else if path.is_file() {
            if path.extension() == Some(OsStr::new("bdf")) && filter(path) {
                self.files.push(FontPath {
                    absolute: path.to_path_buf(),
                    relative: self.prefix.join(file_name),
                });
            }
        } else {
            panic!("path is not a dir or file");
        }

        Ok(())
    }
}

pub fn parse_fonts(path: &Path) -> Result<Vec<FontFile>> {
    parse_fonts_with_filter(path, |_| true)
}

pub fn parse_fonts_with_filter<F: Fn(&Path) -> bool>(
    path: &Path,
    filter: F,
) -> Result<Vec<FontFile>> {
    let paths = DirWalker::new(path, filter).map(|walker| walker.files)?;

    let files = paths
        .into_iter()
        .map(|path| {
            let bdf = std::fs::read(&path.absolute).unwrap();
            let str = String::from_utf8_lossy(&bdf);
            let parsed = Font::parse(&str);

            FontFile { path, parsed }
        })
        .collect::<Vec<_>>();

    Ok(files)
}

pub fn print_parser_result(files: &[FontFile]) -> usize {
    let mut num_errors = 0;

    for font_file in files {
        if font_file.parsed.is_err() {
            num_errors += 1;
        }

        print!("{0: <60}", font_file.path.relative.to_string_lossy());
        match &font_file.parsed {
            Ok(_font) => println!("{}", "OK".green()),
            Err(e) => println!("{} {:}", "Error:".red(), e),
        }
    }

    println!(
        "\n{} out of {} fonts passed ({} failed)\n",
        files.len() - num_errors,
        files.len(),
        num_errors
    );

    num_errors
}

//! Font converter for embedded-graphics.
//!
//! This crate can be used to convert BDF fonts into [`embedded-graphics`] fonts.
//! Two output formats are supported: [`MonoFont`] and [`BdfFont`]. Support for
//! [`MonoFont`]s is included in [`embedded-graphics`] and no additional crates
//! are required. [`BdfFont`]s require an additional dependency on the
//! [`eg-bdf`] crate and have the advantage that proportional fonts are
//! supported.
//!
//! The crate can either be used as a library to convert fonts in a
//! build script or as a command line to convert them ahead of time.
//!
//! # Using `eg_font_converter` in a build script
//!
//! First add the converter to `build.rs`:
//!
//! ```no_run
//! use eg_font_converter::{FontConverter, Mapping};
//!
//! let out_dir = std::env::var_os("OUT_DIR").unwrap();
//!
//! let font_6x10 = FontConverter::with_file("examples/6x10.bdf", "FONT_6X10_AZ")
//!     .glyphs('A'..='Z')
//!     .convert_mono_font()
//!     .unwrap();
//!
//! font_6x10.save(&out_dir).unwrap();
//! ```
//!
//! And then use the [`include!`] macro to import the generated code into your project:
//!
//! ```ignore
//! include!(concat!(env!("OUT_DIR"), "/font_6x10.rs"));
//! ```
//!
//! The font can now be used like any other [`MonoFont`] by using the
//! `FONT_6X10_AZ` constant in a [`MonoTextStyle`].
//!
//! # Using `eg_font_converter` as a command line tool
//!
//! Install the `eg-font-converter` tool using cargo:
//!
//! ```sh
//! cargo install eg-font-converter
//! ```
//! Run `eg-font-converter --help` to see a list of all available options.
//!
//! The tool can now be used to convert BDF fonts:
//! ```sh
//! eg-font-converter --glyph-range A Z --rust font.rs --data font.data 6x10.bdf FONT
//! ```
//! If the `--glyph-range` or `--mapping` options are not specified to limit the
//! selection of glyphs, all glyphs in the source font are included by default.
//!
//! The generated files can be included into a project by using the [`include!`]
//! macro or with a `mod` statement, if the generated files are inside the
//! project's `src` directory.
//!
//! [`embedded-graphics`]: embedded_graphics
//! [`eg-bdf`]: eg_bdf
//! [`MonoFont`]: embedded_graphics::mono_font::MonoFont
//! [`MonoTextStyle`]: embedded_graphics::mono_font::MonoTextStyle
//! [`BdfFont`]: eg_bdf::BdfFont

#![warn(
    missing_docs,
    missing_copy_implementations,
    missing_debug_implementations,
    trivial_numeric_casts,
    unused
)]
#![deny(unsafe_code)]

use anyhow::{anyhow, ensure, Context, Result};
use bdf_parser::{Encoding, Font, Glyph, Property};
use embedded_graphics::mono_font::mapping::GlyphMapping;
use std::{
    collections::BTreeSet,
    iter,
    ops::{Range, RangeInclusive},
    path::{Path, PathBuf},
};

pub use embedded_graphics::mono_font::mapping::Mapping;

mod eg_bdf_font;
mod mono_font;

pub use eg_bdf_font::EgBdfOutput;
pub use mono_font::MonoFontOutput;

#[derive(Debug)]
enum FileOrString<'a> {
    File(PathBuf),
    String(&'a str),
}

/// Font converter.
#[derive(Debug)]
pub struct FontConverter<'a> {
    bdf: FileOrString<'a>,
    name: String,
    file_stem: String, //TODO: make configurable
    replacement_character: Option<char>,
    constant_visibility: Visibility,
    embedded_graphics_crate_path: String,
    data_file_extension: String,
    data_file_path: Option<PathBuf>,
    comments: Vec<String>,

    glyphs: BTreeSet<char>,
    missing_glyph_substitute: Option<char>,
}

impl<'a> FontConverter<'a> {
    /// Creates a font converter from a BDF file.
    pub fn with_file<P: AsRef<Path>>(bdf_file: P, name: &str) -> Self {
        Self::new(FileOrString::File(bdf_file.as_ref().to_owned()), name)
    }

    /// Creates a font converter from BDF data.
    pub fn with_string(bdf: &'a str, name: &str) -> Self {
        Self::new(FileOrString::String(bdf), name)
    }

    fn new(file_or_data: FileOrString<'a>, name: &str) -> Self {
        Self {
            bdf: file_or_data,
            name: name.to_string(),
            file_stem: name.to_ascii_lowercase(),
            replacement_character: None,
            constant_visibility: Visibility::Pub,
            embedded_graphics_crate_path: "embedded_graphics".to_string(),
            data_file_extension: "data".to_string(),
            data_file_path: None,
            comments: Vec::new(),
            glyphs: BTreeSet::new(),
            missing_glyph_substitute: None,
        }
    }

    /// Adds glyphs to the generated font.
    ///
    /// If no specific glyph ranges are provided, all glyphs in the source
    /// BDF data are converted.
    ///
    /// # Examples
    ///
    /// To add a specific subset of glyphs this method can be called with
    /// different argument types:
    ///
    /// ```
    /// # let DATA = "";
    /// use eg_font_converter::FontConverter;
    ///
    /// let converter = FontConverter::with_string(DATA, "FONT")
    ///     .glyphs('a')
    ///     .glyphs('b'..'c')
    ///     .glyphs('d'..='e')
    ///     .glyphs(&['f', 'g'][..])
    ///     .glyphs("hij");
    /// ```
    ///
    /// Use the [`Mapping`] enum to include all glyphs in one of the ISO 8859
    /// encodings, ASCII or JIS X 0201:
    ///
    /// ```
    /// # let DATA = "";
    /// use eg_font_converter::{FontConverter, Mapping};
    ///
    /// let converter = FontConverter::with_string(DATA, "FONT")
    ///     .glyphs(Mapping::Iso8859_15);
    /// ```
    pub fn glyphs<G: GlyphRange>(mut self, glyphs: G) -> Self {
        self.glyphs.extend(glyphs.glyphs());

        self
    }

    /// Sets a substitution glyph for missing glyphs.
    ///
    /// If a substitution glyph is set the converter will replace missing glyphs
    /// in the BDF font by the specified glyph instead of failing with an error.
    pub fn missing_glyph_substitute(mut self, substitute: char) -> Self {
        self.missing_glyph_substitute = Some(substitute);

        self
    }

    /// Sets the replacement character.
    ///
    /// This character will be drawn if the generated font doesn't include a glyph for a character.
    ///
    /// When the character isn't specified by calling this method it will be set
    /// to the first available character of the following fallbacks:
    /// 1. The unicode replacement character: `�` (U+FFFD)
    /// 2. A question mark: `?`
    /// 3. The first glyph in the converted font
    pub fn replacement_character(mut self, replacement_character: char) -> Self {
        self.replacement_character = Some(replacement_character);

        self
    }

    /// Set the visibility of the generated constant.
    ///
    /// Defaults to `pub` ([`Visibility::Pub`]) visibility.
    pub fn constant_visibility(mut self, visibility: Visibility) -> Self {
        self.constant_visibility = visibility;

        self
    }

    /// Sets the type path to the embedded graphics crate.
    ///
    /// The type path is used to import `embedded_graphics` types in generated Rust code. It can be
    /// changed in case the `embedded_graphics` crate is renamed in `Cargo.toml` or the code will be
    /// used inside the `embedded_graphics` crate.
    ///
    /// Defaults to `embedded_graphics`.
    pub fn embedded_graphics_crate_path(mut self, path: &str) -> Self {
        self.embedded_graphics_crate_path = path.to_string();

        self
    }

    /// Sets the file extension for the generated data file.
    ///
    /// Defaults to `data`.
    pub fn data_file_extension(mut self, extension: &str) -> Self {
        self.data_file_extension = extension.to_string();

        self
    }

    /// Sets the path to the data file.
    ///
    /// The data file path is prepended to the filename in `include_bytes` statements in the
    /// generated Rust code.
    ///
    /// By default the data files are expected to be stored in the same directory as the Rust files.
    pub fn data_file_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.data_file_path = Some(path.as_ref().to_owned());

        self
    }

    /// Adds a documentation comments to the generated Rust code.
    pub fn comment(mut self, comment: &str) -> Self {
        self.comments.push(comment.to_string());

        self
    }

    fn convert(&self) -> Result<ConvertedFont> {
        ensure!(
            is_valid_identifier(&self.name),
            "name is not a valid Rust identifier: {}",
            self.name
        );

        let bdf = match &self.bdf {
            FileOrString::File(file) => {
                let data = std::fs::read(file)
                    .with_context(|| format!("couldn't read BDF file from {file:?}"))?;

                let str = String::from_utf8_lossy(&data);
                Font::parse(&str)
            }
            FileOrString::String(str) => Font::parse(str),
        }
        .with_context(|| "couldn't parse BDF file".to_string())?;

        let glyphs = if self.glyphs.is_empty() {
            bdf.glyphs.iter().cloned().collect()
        } else {
            self.glyphs
                .iter()
                .copied()
                .map(|c| {
                    let glyph_c =
                        if bdf.glyphs.get(c).is_none() && self.missing_glyph_substitute.is_some() {
                            self.missing_glyph_substitute.unwrap()
                        } else {
                            c
                        };

                    bdf.glyphs
                        .get(glyph_c)
                        .cloned()
                        .map(|mut glyph| {
                            // replace glyph encoding for substitutes
                            // TODO: assumes unicode
                            glyph.encoding = Encoding::Standard(c as u32);
                            glyph
                        })
                        .ok_or_else(|| {
                            anyhow!(
                                "glyph '{}' (U+{:04X}) is not contained in the BDF font",
                                glyph_c,
                                u32::from(glyph_c)
                            )
                        })
                })
                .collect::<Result<Vec<_>, _>>()?
        };

        // TODO: handle missing (incorrect?) properties
        let ascent = bdf
            .metadata
            .properties
            .try_get::<i32>(Property::FontAscent)
            .ok()
            .filter(|v| *v >= 0)
            .unwrap_or_default() as u32; //TODO: convert to error

        let descent = bdf
            .metadata
            .properties
            .try_get::<i32>(Property::FontDescent)
            .ok()
            .filter(|v| *v >= 0)
            .unwrap_or_default() as u32; //TODO: convert to error

        // TODO: read from BDF and use correct fallbacks (https://www.x.org/docs/XLFD/xlfd.pdf 3.2.30)
        let underline_position = ascent + 1;
        let underline_thickness = 1;
        let strikethrough_position = (ascent + descent) / 2;
        let strikethrough_thickness = 1;

        let mut font = ConvertedFont {
            bdf,
            name: self.name.clone(),
            file_stem: self.file_stem.clone(),
            glyphs,
            replacement_character: 0,
            constant_visibility: self.constant_visibility.clone(),
            embedded_graphics_crate_path: self.embedded_graphics_crate_path.clone(),
            data_file_extension: self.data_file_extension.clone(),
            data_file_path: self.data_file_path.clone(),
            comments: self.comments.clone(),
            ascent,
            descent,
            underline_position,
            underline_thickness,
            strikethrough_position,
            strikethrough_thickness,
        };

        //TODO: add tests
        if let Some(c) = self.replacement_character {
            font.replacement_character = font.glyph_index(c).ok_or_else(|| {
                anyhow!(
                    "replacement character '{}' (U+{:04X}) is not included in the glyphs",
                    c,
                    c as u32
                )
            })?;
        } else if let Some(index) = font
            .glyph_index(char::REPLACEMENT_CHARACTER)
            .or_else(|| font.glyph_index('?'))
        {
            font.replacement_character = index;
        };

        Ok(font)
    }

    /// Converts the font for use with the [`eg-bdf`] crate.
    ///
    /// [`eg-bdf`]: eg_bdf
    pub fn convert_eg_bdf(&self) -> Result<EgBdfOutput> {
        self.convert().and_then(EgBdfOutput::new)
    }

    /// Converts the font for use with [`MonoFont`].
    ///
    /// [`MonoFont`]: embedded_graphics::mono_font::MonoFont
    pub fn convert_mono_font(&self) -> Result<MonoFontOutput> {
        self.convert()
            .and_then(EgBdfOutput::new)
            .and_then(MonoFontOutput::new)
    }
}

fn is_valid_identifier(ident: &str) -> bool {
    ident.starts_with(|c: char| c.is_ascii_alphabetic())
        && ident.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[derive(Debug, PartialEq)]
struct ConvertedFont {
    pub bdf: Font,
    pub name: String,
    pub file_stem: String,
    pub constant_visibility: Visibility,
    pub embedded_graphics_crate_path: String,
    pub data_file_extension: String,
    pub data_file_path: Option<PathBuf>,
    pub comments: Vec<String>,

    pub glyphs: Vec<Glyph>,
    pub replacement_character: usize,

    pub ascent: u32,
    pub descent: u32,

    pub underline_position: u32,
    pub underline_thickness: u32,
    pub strikethrough_position: u32,
    pub strikethrough_thickness: u32,
}

impl ConvertedFont {
    fn glyph_index(&self, c: char) -> Option<usize> {
        // TODO: assumes unicode
        let encoding = Encoding::Standard(c as u32);

        self.glyphs
            .iter()
            .enumerate()
            .find(|(_, glyph)| glyph.encoding == encoding)
            .map(|(index, _)| index)
    }

    fn rust_file_path(&self, output_directory: &Path) -> PathBuf {
        output_directory.join(&self.file_stem).with_extension("rs")
    }

    fn data_file(&self) -> PathBuf {
        let file: &Path = self.file_stem.as_ref();
        let file = file.with_extension(&self.data_file_extension);

        if let Some(path) = &self.data_file_path {
            path.join(file)
        } else {
            file
        }
    }

    fn data_file_path(&self, output_directory: &Path) -> PathBuf {
        output_directory.join(self.data_file())
    }
}

impl GlyphMapping for ConvertedFont {
    fn index(&self, c: char) -> usize {
        // TODO: assumes unicode
        let encoding = Encoding::Standard(c as u32);

        // TODO: support replacement character
        self.glyphs
            .iter()
            .enumerate()
            .find(|(_, glyph)| glyph.encoding == encoding)
            .map(|(index, _)| index)
            .unwrap_or_default()
    }
}

/// Glyph range.
///
/// See [`FontConverter::glyphs`] for more information.
pub trait GlyphRange {
    /// Returns an iterator over all glyphs in this range.
    fn glyphs(self) -> Box<dyn Iterator<Item = char>>;
}

impl GlyphRange for char {
    fn glyphs(self) -> Box<dyn Iterator<Item = char>> {
        Box::new(iter::once(self))
    }
}

impl GlyphRange for &[char] {
    fn glyphs(self) -> Box<dyn Iterator<Item = char>> {
        Box::new(Vec::from(self).into_iter())
    }
}

impl GlyphRange for RangeInclusive<char> {
    fn glyphs(self) -> Box<dyn Iterator<Item = char>> {
        Box::new(self.clone())
    }
}

impl GlyphRange for Range<char> {
    fn glyphs(self) -> Box<dyn Iterator<Item = char>> {
        Box::new(self.clone())
    }
}

impl GlyphRange for &str {
    fn glyphs(self) -> Box<dyn Iterator<Item = char>> {
        Box::new(self.chars().collect::<Vec<_>>().into_iter())
    }
}

impl GlyphRange for Mapping {
    fn glyphs(self) -> Box<dyn Iterator<Item = char>> {
        Box::new(self.glyph_mapping().chars().collect::<Vec<_>>().into_iter())
    }
}

/// Constant visibility.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Visibility {
    /// No explicit visibility.
    Private,
    /// `pub` visibility.
    Pub,
    /// `pub(crate)` visibility.
    PubCrate,
    /// `pub(self)` visibility.
    PubSelf,
    /// `pub(super)` visibility.
    PubSuper,
    /// `pub(in path)` visibility.
    PubIn(String),
}

impl Visibility {
    // TODO: is Visibility even used anymore?
    #[allow(unused)]
    fn to_rust(&self) -> String {
        match self {
            Visibility::Private => "",
            Visibility::Pub => "pub",
            Visibility::PubCrate => "pub(crate)",
            Visibility::PubSelf => "pub(self)",
            Visibility::PubSuper => "pub(super)",
            Visibility::PubIn(path) => return format!("pub(in {path})"),
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FONT: &str = r#"
        STARTFONT 2.1
        FONT -gbdfed-Unknown-Medium-R-Normal--16-120-96-96-P-100-FontSpecific-0
        SIZE 8 96 96
        FONTBOUNDINGBOX 8 8 0 0
        STARTPROPERTIES 9
        POINT_SIZE 120
        PIXEL_SIZE 8
        RESOLUTION_X 96
        RESOLUTION_Y 96
        FONT_ASCENT 8
        FONT_DESCENT 0
        AVERAGE_WIDTH 100
        SPACING "C"
        _GBDFED_INFO "Edited with gbdfed 1.6."
        ENDPROPERTIES
        CHARS 1
        STARTCHAR A
        ENCODING 65
        SWIDTH 750 0
        DWIDTH 8 0
        BBX 8 8 0 0
        BITMAP
        FF
        81
        81
        81
        81
        81
        81
        FF
        ENDCHAR
        ENDFONT
    "#;

    #[test]
    fn with_string() {
        FontConverter::with_string(FONT, "TEST")
            .glyphs('A'..='A')
            .convert()
            .unwrap();
    }

    #[test]
    fn add_glyphs() {
        let converter = FontConverter::with_string(FONT, "TEST")
            .glyphs('E')
            .glyphs('A'..='C')
            .glyphs('Y'..'Z')
            .glyphs("HG")
            .glyphs(&['W', 'V'] as &[_]);

        assert!(converter
            .glyphs
            .iter()
            .eq(['A', 'B', 'C', 'E', 'G', 'H', 'V', 'W', 'Y'].iter()));
    }

    #[test]
    fn no_glyph_ranges() {
        let converter = FontConverter::with_string(FONT, "TEST");
        let font = converter.convert().unwrap();
        assert_eq!(font.glyphs.len(), 1);
        assert_eq!(font.glyphs[0].name, "A");
        assert_eq!(font.glyphs[0].encoding, Encoding::Standard(65));
    }
}

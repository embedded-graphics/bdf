use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use clap::{command, Parser};
use eg_font_converter::FontConverter;
use embedded_graphics::mono_font::mapping::Mapping;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// BDF file.
    #[arg(required_unless_present = "list_mappings")]
    bdf_file: Option<PathBuf>,

    /// Name of the Rust constant.
    #[arg(required_unless_present = "list_mappings")]
    name: Option<String>,

    /// Inline PNG image in documentation.
    #[arg(long)]
    inline_png: bool,

    /// Generate Rust file.
    #[arg(long)]
    rust: Option<PathBuf>,

    /// Generate data file.
    #[arg(long)]
    data: Option<PathBuf>,

    /// Generate PNG image file.
    #[arg(long)]
    png: Option<PathBuf>,

    /// Limit the selection of glyphs to those included in the provided mapping.
    #[arg(long, value_parser = parse_mapping)]
    mapping: Option<Mapping>,

    /// Limit the selection of glyphs to the given inclusive range.
    #[arg(long, num_args = 2, id = "char", conflicts_with = "mapping")]
    glyph_range: Vec<char>,

    /// Type path to the embedded-graphics crate
    #[arg(long, default_value = "::embedded_graphics")]
    embedded_graphics_crate_path: String,

    /// List supported mappings.
    #[arg(long)]
    list_mappings: bool,

    /// Substitution character for missing characters in the BDF file.
    #[arg(long)]
    missing_glyph_substitute: Option<char>,

    /// Add a documentation comment to the generated Rust code.
    #[arg(long)]
    comment: Vec<String>,
}

fn parse_mapping(s: &str) -> Result<Mapping> {
    Mapping::iter()
        .find(|m| m.mime() == s)
        .ok_or_else(|| anyhow!("use --list-mappings to show all available mappings"))
}

fn list_mappings() {
    println!("Supported mappings:");

    for mapping in Mapping::iter() {
        println!("  {}", mapping.mime());
    }
}

fn convert(args: &Args) -> Result<()> {
    let bdf_file = args.bdf_file.as_ref().unwrap();
    let name = args.name.as_ref().unwrap();

    let mut converter = FontConverter::new(bdf_file, name)
        .embedded_graphics_crate_path(&args.embedded_graphics_crate_path);

    if let Some(mapping) = args.mapping {
        converter = converter.glyphs(mapping);
    } else if !args.glyph_range.is_empty() {
        for range in args.glyph_range.chunks(2) {
            converter = converter.glyphs(range[0]..=range[1]);
        }
    }

    if let Some(substitute) = args.missing_glyph_substitute {
        converter = converter.missing_glyph_substitute(substitute);
    }
    //.inline_png(args.inline_png);

    let font = converter.convert_mono_font()?;

    //TODO: use FontConverterOutput::save

    if let Some(rust) = &args.rust {
        std::fs::write(rust, font.rust())
            .with_context(|| format!("Failed to write Rust file {}", rust.to_string_lossy()))?;
    }

    if let Some(data) = &args.data {
        std::fs::write(data, font.data())
            .with_context(|| format!("Failed to write data file {}", data.to_string_lossy()))?;
    }

    if let Some(png) = &args.png {
        font.save_png(png)?;
    }

    Ok(())
}

fn main() {
    let args = Args::parse();

    if args.list_mappings {
        list_mappings();
        return;
    }

    if let Err(e) = convert(&args) {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

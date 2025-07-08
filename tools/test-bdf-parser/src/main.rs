use clap::Parser;
use eg_bdf::BdfTextStyle;
use eg_font_converter::FontConverter;
use embedded_graphics::{
    mono_font::MonoTextStyle,
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Line, PrimitiveStyle, StyledDrawable},
    text::{renderer::TextRenderer, Baseline, Text},
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay};
use owo_colors::OwoColorize;
use std::{fs, path::PathBuf};

use test_bdf_parser::*;

#[derive(Parser)]
struct Arguments {
    /// Output directory for font specimens in PNG format.
    #[arg(long)]
    png_out: Option<PathBuf>,

    /// Output scale for PNG images.
    #[arg(long, default_value = "1")]
    png_scale: u32,

    /// Path to a BDF file or a directory containing BDF files.
    file_or_directory: PathBuf,
}

fn draw_specimen(style: impl TextRenderer<Color = Rgb888> + Copy) -> SimulatorDisplay<Rgb888> {
    let single_line = Text::with_baseline(
        "The quick brown fox jumps over the lazy dog.",
        Point::zero(),
        style,
        Baseline::Top,
    );

    // 10 px minimum line height to ensure output even if metrics are wrong.
    let single_line_height = single_line.bounding_box().size.height.max(10);

    let display_height = single_line_height * 3;
    let display_width = (single_line.bounding_box().size.width + 10).max(display_height);

    let text_position = Point::new(5, single_line_height as i32);

    let mut display = SimulatorDisplay::<Rgb888>::new(Size::new(display_width, display_height));

    // Draw baseline grid

    for offset in [Point::zero(), Point::new(0, single_line_height as i32)] {
        Line::with_delta(
            text_position.y_axis() + offset,
            Point::new(display_width as i32, 0),
        )
        .draw_styled(
            &PrimitiveStyle::with_stroke(Rgb888::CSS_DARK_SLATE_GRAY, 1),
            &mut display,
        )
        .unwrap();
    }

    // Draw marker for X start position

    Line::with_delta(text_position.x_axis(), Point::new(0, display_height as i32))
        .draw_styled(
            &PrimitiveStyle::with_stroke(Rgb888::CSS_DARK_SLATE_GRAY, 1),
            &mut display,
        )
        .unwrap();

    let text = Text::new(
        "The quick brown fox jumps over the lazy dog.\n0123456789",
        text_position,
        style,
    );

    // Draw bounding box

    text.bounding_box()
        .draw_styled(
            &PrimitiveStyle::with_stroke(Rgb888::CSS_LIGHT_SLATE_GRAY, 1),
            &mut display,
        )
        .unwrap();

    text.draw(&mut display).unwrap();

    display
}

pub fn main() {
    let args: Arguments = Arguments::parse();

    let fonts = parse_fonts(&args.file_or_directory).expect("Could not parse fonts");
    let num_errors = print_parser_result(&fonts);

    let output_settings = OutputSettingsBuilder::new().scale(args.png_scale).build();

    if let Some(png_directory) = args.png_out {
        for file in fonts.iter().filter(|file| file.parsed.is_ok()) {
            println!(
                "Generating specimen: {}",
                file.path.relative.to_string_lossy()
            );

            let output_file = png_directory.join(&file.path.relative);
            let output_dir = output_file.parent().unwrap();

            fs::create_dir_all(output_dir).unwrap();

            let converter = FontConverter::with_file(&file.path.absolute, "FONT");

            match converter.convert_eg_bdf() {
                Ok(converted_bdf) => {
                    let bdf_specimen =
                        draw_specimen(BdfTextStyle::new(&converted_bdf.as_font(), Rgb888::WHITE));
                    bdf_specimen
                        .to_rgb_output_image(&output_settings)
                        .save_png(output_file.with_extension("bdf.png"))
                        .unwrap();
                }
                Err(e) => println!("{} {e}", "Error (eg-bdf):".red()),
            };

            match converter.convert_mono_font() {
                Ok(converted_mono) => {
                    let mono_specimen =
                        draw_specimen(MonoTextStyle::new(&converted_mono.as_font(), Rgb888::WHITE));
                    mono_specimen
                        .to_rgb_output_image(&output_settings)
                        .save_png(output_file.with_extension("mono.png"))
                        .unwrap();
                }
                Err(e) => println!("{} {e}", "Error (mono):".red()),
            };
        }
    }

    assert_eq!(num_errors, 0, "Not all font files parsed successfully");
}

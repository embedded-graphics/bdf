use anyhow::{anyhow, Context, Result};
use eg_bdf::BdfTextStyle;
use eg_font_converter::{FontConverter, Mapping};
use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Line, PrimitiveStyle},
    text::{Baseline, Text},
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};

fn main() {
    if let Err(e) = try_main() {
        eprintln!("Error: {e:#}");
    }
}

fn draw_text(display: &mut SimulatorDisplay<Rgb888>, text: &Text<BdfTextStyle<Rgb888>>) -> Point {
    text.bounding_box()
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::CSS_DARK_ORANGE, 1))
        .draw(display)
        .unwrap();

    text.draw(display).unwrap()
}

fn try_main() -> Result<()> {
    let file = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow!("missing filename"))?;

    let output = FontConverter::with_file(&file, "BDF_FILE")
        .glyphs(Mapping::Ascii)
        .missing_glyph_substitute('?')
        .convert_eg_bdf()
        .with_context(|| "couldn't convert font")?;

    let font = output.as_font();
    let style = BdfTextStyle::new(&font, Rgb888::WHITE);

    let settings = OutputSettingsBuilder::new().scale(3).build();
    let mut window = Window::new("Font viewer", &settings);

    let mut display = SimulatorDisplay::<Rgb888>::new(Size::new(600, 200));

    let abc = ('a'..='z').collect::<String>();
    let digits = ('0'..='9').collect::<String>();
    let string = format!(
        "The quick brown fox jumps over the lazy dog\n{}\n{}\n{}",
        abc,
        abc.to_ascii_uppercase(),
        digits
    );

    let position = Point::new(20, 20);

    Line::with_delta(position.y_axis(), Point::zero() + display.size().x_axis())
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::CSS_DIM_GRAY, 1))
        .draw(&mut display)
        .unwrap();

    let text = Text::new(&string, position, style);
    draw_text(&mut display, &text);

    let position = Point::new(20, 150);

    Line::with_delta(position.y_axis(), Point::zero() + display.size().x_axis())
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::CSS_DIM_GRAY, 1))
        .draw(&mut display)
        .unwrap();

    let p = draw_text(
        &mut display,
        &Text::with_baseline("Top ", position, style, Baseline::Top),
    );

    let p = draw_text(
        &mut display,
        &Text::with_baseline(
            "Middle ",
            Point::new(p.x, position.y),
            style,
            Baseline::Middle,
        ),
    );

    let p = draw_text(
        &mut display,
        &Text::with_baseline(
            "Bottom ",
            Point::new(p.x, position.y),
            style,
            Baseline::Bottom,
        ),
    );

    draw_text(
        &mut display,
        &Text::with_baseline(
            "Alphabetic",
            Point::new(p.x, position.y),
            style,
            Baseline::Alphabetic,
        ),
    );

    window.show_static(&display);

    Ok(())
}

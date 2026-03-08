use anyhow::{anyhow, Context, Result};
use eg_bdf::BdfTextStyle;
use eg_font_converter::{FontConverter, Mapping};
use embedded_graphics::{
    geometry::AnchorPoint,
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::{Gray8, Rgb888},
    prelude::*,
    primitives::{Line, PrimitiveStyle},
    text::{renderer::TextRenderer, Alignment, Baseline, Text, TextStyleBuilder},
};
use embedded_graphics_simulator::{
    sdl2::Keycode, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};

fn main() {
    if let Err(e) = try_main() {
        eprintln!("Error: {e:#}");
    }
}

fn draw_character_bounding_boxes<S: TextRenderer<Color = Rgb888>>(
    display: &mut SimulatorDisplay<Rgb888>,
    text: &Text<S>,
) where
    S: Clone,
{
    let mut dummy_display = display.clone();

    let mut p = text.position;
    let mut toggle = false;

    for c in text.text.chars().map(|c| c.to_string()) {
        if c == "\n" {
            p.x = text.position.x;
        }

        let t = Text {
            text: &c,
            position: p,
            ..text.clone()
        };

        let fill_color = Gray8::new(if toggle { 30 } else { 50 }).into();

        t.bounding_box()
            .into_styled(PrimitiveStyle::with_fill(fill_color))
            .draw(display)
            .unwrap();

        p = t.draw(&mut dummy_display).unwrap();
        toggle = !toggle;
    }
}

/// Draws a text and its bounding box.
fn draw_text<S: TextRenderer<Color = Rgb888>>(
    display: &mut SimulatorDisplay<Rgb888>,
    text: &Text<S>,
) -> Point {
    text.bounding_box()
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::CSS_DARK_ORANGE, 1))
        .draw(display)
        .unwrap();

    text.draw(display).unwrap()
}

fn draw<S: TextRenderer<Color = Rgb888> + Copy>(
    display: &mut SimulatorDisplay<Rgb888>,
    style: S,
    line_height: u32,
    highlight_characters: bool,
) {
    let abc = ('a'..='z').collect::<String>();
    let digits = ('0'..='9').collect::<String>();
    let string = format!(
        "The quick brown fox jumps over the lazy dog\n{}\n{}\n{}",
        abc,
        abc.to_ascii_uppercase(),
        digits
    );

    let position = Point::new(5, 5 + line_height as i32);

    Line::with_delta(position.y_axis(), Point::zero() + display.size().x_axis())
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::CSS_DIM_GRAY, 1))
        .draw(display)
        .unwrap();

    let text = Text::new(&string, position, style);
    if highlight_characters {
        draw_character_bounding_boxes(display, &text);
    }
    draw_text(display, &text);

    let position = display.bounding_box().anchor_point(AnchorPoint::BottomLeft)
        + Point::new(5, -(line_height as i32) * 3 / 2);

    Line::with_delta(position.y_axis(), Point::zero() + display.size().x_axis())
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::CSS_DIM_GRAY, 1))
        .draw(display)
        .unwrap();

    let p = draw_text(
        display,
        &Text::with_baseline("Top ", position, style, Baseline::Top),
    );

    let p = draw_text(
        display,
        &Text::with_baseline(
            "Middle ",
            Point::new(p.x, position.y),
            style,
            Baseline::Middle,
        ),
    );

    let p = draw_text(
        display,
        &Text::with_baseline(
            "Bottom ",
            Point::new(p.x, position.y),
            style,
            Baseline::Bottom,
        ),
    );

    draw_text(
        display,
        &Text::with_baseline(
            "Alphabetic",
            Point::new(p.x, position.y),
            style,
            Baseline::Alphabetic,
        ),
    );
}

fn try_main() -> Result<()> {
    let file = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow!("missing filename"))?;

    let converter = FontConverter::with_file(&file, "BDF_FILE")
        .glyphs(Mapping::Ascii)
        .missing_glyph_substitute('?');

    let bdf_output = converter
        .convert_eg_bdf()
        .with_context(|| "couldn't convert font")?;
    let bdf_font = bdf_output.as_font();

    let mono_output = converter
        .convert_mono_font()
        .with_context(|| "couldn't convert font")?;
    let mono_font = mono_output.as_font();

    let hints_style = MonoTextStyle::new(&FONT_6X10, Rgb888::CSS_DIM_GRAY);
    let bottom_right = TextStyleBuilder::new()
        .baseline(Baseline::Bottom)
        .alignment(Alignment::Right)
        .build();

    let line_height = bdf_font.ascent + bdf_font.descent;
    let display_height = line_height * 8;
    let display_width = (line_height * 25).max(display_height);
    let display_size = Size::new(display_width, display_height);

    let mut display = SimulatorDisplay::<Rgb888>::new(display_size);

    let scale = (1200 / display_size.width).max(1);

    let settings = OutputSettingsBuilder::new().scale(scale).build();
    let mut window = Window::new("Font viewer", &settings);

    let mut use_mono_font = false;
    let mut highlight_characters = false;

    'main_loop: loop {
        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::M => use_mono_font = !use_mono_font,
                    Keycode::C => highlight_characters = !highlight_characters,
                    Keycode::Q => break 'main_loop,
                    _ => {}
                },
                SimulatorEvent::Quit => break 'main_loop,
                _ => {}
            }
        }

        let mut hint = "Press M to toggle".to_string();

        display.clear(Rgb888::BLACK).unwrap();
        if use_mono_font {
            let style = MonoTextStyle::new(&mono_font, Rgb888::WHITE);
            draw(&mut display, style, line_height, highlight_characters);

            hint.insert_str(0, "Mono | ");
        } else {
            let style = BdfTextStyle::new(&bdf_font, Rgb888::WHITE);
            draw(&mut display, style, line_height, highlight_characters);

            hint.insert_str(0, "Bdf | ");
        }

        let corner = display
            .bounding_box()
            .offset(-3)
            .anchor_point(AnchorPoint::BottomRight);
        Text::with_text_style(&hint, corner, hints_style, bottom_right)
            .draw(&mut display)
            .unwrap();
    }

    Ok(())
}

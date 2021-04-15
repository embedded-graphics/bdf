use eg_bdf::{include_bdf, text::BdfTextStyle, BdfFont};
use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
    text::{Alignment, Text},
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};

const FONT_6X10: BdfFont =
    include_bdf!("examples/6x10.bdf", 'A'..='Z' | 'a'..='z' | '0'..='9' | ' ');
const FONT_10X20: BdfFont = include_bdf!("examples/10x20.bdf");

fn main() -> Result<(), std::convert::Infallible> {
    let mut display = SimulatorDisplay::<Rgb888>::new(Size::new(400, 150));

    let style_small = BdfTextStyle::new(&FONT_6X10, Rgb888::RED);
    let style_large = BdfTextStyle::new(&FONT_10X20, Rgb888::GREEN);

    Text::new("Hello BDF! äöü,\"#", Point::new(30, 50), style_large).draw(&mut display)?;

    Text::new("A\nB\nC", Point::new(10, 50), style_large).draw(&mut display)?;

    Text::with_alignment(
        "Hello BDF! äöü,\"#",
        Point::new(150, 100),
        style_large,
        Alignment::Center,
    )
    .draw(&mut display)?;

    Text::with_alignment(
        "Line 1\nLine 2\nLast line",
        Point::new(390, 10),
        style_small,
        Alignment::Right,
    )
    .draw(&mut display)
    .unwrap();

    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    Window::new("BDF Font", &output_settings).show_static(&display);

    Ok(())
}

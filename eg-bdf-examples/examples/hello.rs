use eg_bdf::BdfTextStyle;
use embedded_graphics::{
    mono_font::MonoTextStyle,
    pixelcolor::Rgb888,
    prelude::*,
    text::{Alignment, Text},
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};

include!(concat!(env!("OUT_DIR"), "/font_6x10.rs"));
include!(concat!(env!("OUT_DIR"), "/font_10x20.rs"));
include!(concat!(env!("OUT_DIR"), "/font_6x10_mono.rs"));
include!(concat!(env!("OUT_DIR"), "/font_10x20_mono.rs"));

fn main() -> Result<(), std::convert::Infallible> {
    let mut display = SimulatorDisplay::<Rgb888>::new(Size::new(400, 150));

    //TODO: demonstrate BDF and Mono fonts
    let style_small = BdfTextStyle::new(&FONT_6X10, Rgb888::RED);
    let style_large = BdfTextStyle::new(&FONT_10X20, Rgb888::GREEN);
    let style_small_mono = MonoTextStyle::new(&FONT_6X10_MONO, Rgb888::RED);
    let style_large_mono = MonoTextStyle::new(&FONT_10X20_MONO, Rgb888::GREEN);

    Text::new("Hello BDF! äöü€,\"#", Point::new(30, 50), style_large).draw(&mut display)?;
    Text::new("Hello BDF! äöü€,\"#", Point::new(30, 70), style_large_mono).draw(&mut display)?;

    Text::new("A\nB\nC", Point::new(10, 50), style_large).draw(&mut display)?;

    Text::with_alignment(
        "Hello BDF! äöü€,\"#",
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

    Text::with_alignment(
        "Line 1\nLine 2\nLast line",
        Point::new(390, 60),
        style_small_mono,
        Alignment::Right,
    )
    .draw(&mut display)
    .unwrap();

    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    Window::new("BDF Font", &output_settings).show_static(&display);

    Ok(())
}

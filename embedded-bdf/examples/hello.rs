use embedded_bdf::{include_bdf, text::BdfTextStyle, BdfFont};
use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
    text::{HorizontalAlignment, Text, TextStyleBuilder},
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};

const FONT_6X10: BdfFont = include_bdf!("examples/6x10.bdf");
const FONT_10X20: BdfFont = include_bdf!("examples/10x20.bdf", 'A'..='Z' | 'a'..='z' | ' ');

fn main() {
    let mut display = SimulatorDisplay::<Rgb888>::new(Size::new(400, 150));

    let style_small = BdfTextStyle::new(&FONT_6X10, Rgb888::RED);
    let style_large = BdfTextStyle::new(&FONT_10X20, Rgb888::GREEN);

    let text = Text::new("Hello BDF! äöü,\"#", Point::new(10, 50));
    text.into_styled(style_large).draw(&mut display).unwrap();

    let text = Text::new("Hello BDF! äöü,\"#", Point::new(150, 100));
    let style = TextStyleBuilder::new()
        .character_style(style_large)
        .horizontal_alignment(HorizontalAlignment::Center)
        .build();
    text.into_styled(style).draw(&mut display).unwrap();

    let text = Text::new("Line 1\nLine 2\nLast line", Point::new(390, 10));
    let style = TextStyleBuilder::new()
        .character_style(style_small)
        .horizontal_alignment(HorizontalAlignment::Right)
        .build();
    text.into_styled(style).draw(&mut display).unwrap();

    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    Window::new("BDF Font", &output_settings).show_static(&display);
}

use embedded_bdf::{include_bdf, text::BdfTextStyle, BdfFont};
use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
    text::{Text, TextStyleBuilder},
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};

const TIM_R24: BdfFont = include_bdf!("../bdf-parser/tests/u8g2/tools/font/bdf/timR24.bdf");
const HELV_B24: BdfFont = include_bdf!(
    "../bdf-parser/tests/u8g2/tools/font/bdf/helvB24.bdf",
    'A'..='Z' | 'a'..='z' | ' '
);
const HELV_R08: BdfFont = include_bdf!(
    "../bdf-parser/tests/u8g2/tools/font/bdf/helvR08.bdf",
    'A'..='Z' | 'a'..='z' | ' ' | '0'..='9'
);

fn main() {
    let mut display = SimulatorDisplay::<Rgb888>::new(Size::new(400, 150));

    let style1 = BdfTextStyle::new(&TIM_R24, Rgb888::RED);
    let style2 = BdfTextStyle::new(&HELV_B24, Rgb888::GREEN);
    let style3 = BdfTextStyle::new(&HELV_R08, Rgb888::WHITE);

    let text = Text::new("Hello BDF! äöü,\"#", Point::new(10, 50));
    text.into_styled(style1).draw(&mut display).unwrap();

    let text = Text::new("Hello BDF! äöü,\"#", Point::new(150, 100));
    let style = TextStyleBuilder::new()
        .character_style(style2)
        .horizontal_alignment(embedded_graphics::text::HorizontalAlignment::Center)
        .build();
    text.into_styled(style).draw(&mut display).unwrap();

    let text = Text::new("Line 1\nLine 2\nLast line", Point::new(390, 10));
    let style = TextStyleBuilder::new()
        .character_style(style3)
        .horizontal_alignment(embedded_graphics::text::HorizontalAlignment::Right)
        .build();
    text.into_styled(style).draw(&mut display).unwrap();

    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    Window::new("BDF Font", &output_settings).show_static(&display);
}

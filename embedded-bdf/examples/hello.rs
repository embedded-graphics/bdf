use embedded_bdf::{include_bdf, text::BdfStyled, text::BdfTextStyle, BdfFont};
use embedded_graphics::{fonts::Text, pixelcolor::Rgb888, prelude::*};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};

const TIM_R24: BdfFont = include_bdf!("../bdf-parser/tests/u8g2/tools/font/bdf/timR24.bdf");
const HELV_B24: BdfFont = include_bdf!(
    "../bdf-parser/tests/u8g2/tools/font/bdf/helvB24.bdf",
    'A'..='Z' | 'a'..='z' | ' '
);

fn main() {
    let mut display = SimulatorDisplay::<Rgb888>::new(Size::new(300, 150));

    let style1 = BdfTextStyle::new(&TIM_R24, Rgb888::RED);
    let style2 = BdfTextStyle::new(&HELV_B24, Rgb888::GREEN);

    let text = Text::new("Hello BDF! äöü,\"#", Point::new(10, 50));
    BdfStyled::new(text, style1).draw(&mut display).unwrap();

    let text = Text::new("Hello BDF! äöü,\"#", Point::new(10, 100));
    BdfStyled::new(text, style2).draw(&mut display).unwrap();

    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    Window::new("BDF Font", &output_settings).show_static(&display);
}

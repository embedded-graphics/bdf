#![feature(test)]

extern crate bdf_parser;
extern crate test;

use bdf_parser::*;

pub fn add_two(a: i32) -> i32 {
    a + 2
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn parse_single_char_font(b: &mut Bencher) {
        let chardata = r#"STARTFONT 2.1
FONT "a"
SIZE 16 75 75
FONTBOUNDINGBOX 16 16 0 0
STARTPROPERTIES 3
PROP 0
ENDPROPERTIES
STARTCHAR 000
ENCODING 64
DWIDTH 8 0
BBX 8 8 0 0
BITMAP
1f
01
ENDCHAR
ENDFONT
"#;

        b.iter(|| {
            let parser = BdfParser::from_str(&chardata);

            let _out = parser.parse();

            "use me"
        });
    }
}

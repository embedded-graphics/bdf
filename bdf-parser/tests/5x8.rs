extern crate bdf_parser;
extern crate nom;

use bdf_parser::*;
use nom::*;

#[test]
fn it_parses_the_5x8_font() {
    let bdf = include_str!("./5x8.bdf");

    let parser = BDFParser::new(&bdf);

    let out = parser.parse();

    match out {
        Ok((rest, parsed)) => {
            println!("Rest: {:?}", rest);
        }
        Err(err) => {
            match err {
                nom::Err::Error(Context::Code(c, thing)) => println!(
                    "ContextError {:?} {:?}",
                    String::from_utf8(c.to_vec()).unwrap(),
                    thing
                ),
                _ => println!("OtherError {:?}", err),
            };

            panic!();
        }
    }
}

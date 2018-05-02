extern crate bdf_parser;
extern crate nom;

use bdf_parser::*;
use nom::*;

#[test]
fn it_parses_a_single_character() {
    let bdf = include_str!("./a.bdf");

    let parser = BDFParser::new(&bdf);

    let out = parser.parse();

    match out {
        Ok((rest, parsed)) => {
            println!("Rest: {:?}, parsed: {:?}", rest, parsed);
        }
        Err(err) => {
            match err {
                nom::Err::Error(Context::Code(c, _)) => {
                    println!("ContextError {:?}", String::from_utf8(c.to_vec()).unwrap())
                }
                _ => println!("OtherError {:?}", err),
            };

            panic!();
        }
    }
}

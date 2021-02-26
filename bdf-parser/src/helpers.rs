use std::char::REPLACEMENT_CHARACTER;

use bstr::ByteSlice;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while},
    character::complete::{digit1, line_ending, one_of, space0, space1},
    combinator::{eof, map, map_opt, opt, recognize},
    multi::many0,
    sequence::{delimited, preceded},
    IResult, ParseTo,
};

pub fn parse_to_i32(input: &[u8]) -> IResult<&[u8], i32> {
    map_opt(
        recognize(preceded(opt(one_of("+-")), digit1)),
        |i: &[u8]| i.parse_to(),
    )(input)
}

pub fn parse_to_u32(input: &[u8]) -> IResult<&[u8], u32> {
    map_opt(recognize(digit1), |i: &[u8]| i.parse_to())(input)
}

fn comment(input: &[u8]) -> IResult<&[u8], String> {
    map_opt(
        delimited(
            tag(b"COMMENT"),
            opt(preceded(space1, take_until("\n"))),
            line_ending,
        ),
        |c: Option<&[u8]>| c.map_or(Some(String::from("")), |c| c.parse_to()),
    )(input)
}

pub fn skip_comments<'a, F, O>(inner: F) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], O>
where
    F: FnMut(&'a [u8]) -> IResult<&'a [u8], O>,
{
    delimited(many0(comment), inner, many0(comment))
}

pub fn parse_string(input: &[u8]) -> IResult<&[u8], String> {
    map(take_until_line_ending, ascii_to_string_lossy)(input)
}

/// Converts a byte slice into a valid UTF-8 string.
pub fn ascii_to_string_lossy(bytes: &[u8]) -> String {
    bytes
        .iter()
        .copied()
        .map(|byte| {
            if byte >= 0x80 {
                REPLACEMENT_CHARACTER
            } else {
                byte as char
            }
        })
        .collect()
}

fn take_until_line_ending(input: &[u8]) -> IResult<&[u8], &[u8]> {
    take_while(|c| c != b'\n' && c != b'\r')(input)
}

pub fn statement<'a, F, O>(
    keyword: &'a str,
    mut parameters: F,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], O>
where
    F: FnMut(&'a [u8]) -> IResult<&'a [u8], O>,
{
    move |input: &[u8]| {
        let (input, _) = tag(keyword)(input)?;
        let (input, p) = alt((preceded(space1, take_until_line_ending), space0))(input)?;
        let (input, _) = opt(line_ending)(input)?;

        let (inner_input, output) = parameters(p.trim())?;
        eof(inner_input)?;

        Ok((input, output))
    }
}

#[cfg(test)]
#[macro_use]
mod tests {
    use nom::combinator::eof;

    use super::*;

    /// Asserts that a parsing function has returned `Ok` and no remaining input.
    macro_rules! assert_parser_ok {
        ($left:expr, $right:expr) => {
            assert_eq!($left, Ok((&[] as &[u8], $right)));
        };
    }

    #[test]
    fn it_takes_until_any_line_ending() {
        assert_eq!(
            take_until_line_ending(b"Unix line endings\n"),
            Ok(("\n".as_ref(), "Unix line endings".as_ref()))
        );

        assert_eq!(
            take_until_line_ending(b"Windows line endings\r\n"),
            Ok(("\r\n".as_ref(), "Windows line endings".as_ref()))
        );
    }

    #[test]
    fn parse_statement_without_parameters() {
        assert_eq!(
            statement("KEYWORD", eof)(b"KEYWORD"),
            Ok((b"".as_ref(), b"".as_ref()))
        );

        assert_eq!(
            statement("KEYWORD", eof)(b"KEYWORD\nABC"),
            Ok((b"ABC".as_ref(), b"".as_ref()))
        );
    }

    #[test]
    fn parse_statement_with_parameters() {
        assert_eq!(
            statement("KEYWORD", parse_string)(b"KEYWORD param"),
            Ok((b"".as_ref(), "param".to_string())),
        );

        assert_eq!(
            statement("KEYWORD", parse_string)(b"KEYWORD    param   \nABC"),
            Ok((b"ABC".as_ref(), "param".to_string())),
        );
    }

    #[test]
    fn parse_comment() {
        assert_parser_ok!(comment(b"COMMENT test text\n"), "test text".to_string());
    }

    #[test]
    fn parse_empty_comment() {
        assert_parser_ok!(comment(b"COMMENT\n"), String::new());
    }

    #[test]
    fn parse_string_ascii() {
        assert_eq!(
            parse_string(b"Test\n"),
            Ok((b"\n".as_ref(), "Test".to_string()))
        );
    }

    #[test]
    fn parse_string_invalid_ascii() {
        assert_eq!(
            parse_string(b"Abc\x80\n"),
            Ok((b"\n".as_ref(), "Abc\u{FFFD}".to_string()))
        );
    }
}

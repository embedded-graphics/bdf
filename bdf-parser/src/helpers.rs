use nom::{
    bytes::complete::{tag, take_until, take_while},
    character::complete::{digit1, line_ending, multispace0, one_of, space0, space1},
    combinator::{map_opt, opt, recognize},
    multi::many0,
    sequence::{delimited, preceded, separated_pair},
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
            tag("COMMENT"),
            opt(preceded(space1, take_until("\n"))),
            line_ending,
        ),
        |c: Option<&[u8]>| c.map_or(Some(String::from("")), |c| c.parse_to()),
    )(input)
}

pub fn optional_comments(input: &[u8]) -> IResult<&[u8], Vec<String>> {
    preceded(multispace0, many0(comment))(input)
}

pub fn numchars(input: &[u8]) -> IResult<&[u8], u32> {
    preceded(
        space0,
        preceded(tag("CHARS"), preceded(space0, parse_to_u32)),
    )(input)
}

pub fn take_until_line_ending(input: &[u8]) -> IResult<&[u8], &[u8]> {
    take_while(|c| c != b'\n' && c != b'\r')(input)
}

pub fn statement<'a, O, F>(
    keyword: &'a str,
    parameters: F,
) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], O>
where
    F: Fn(&'a [u8]) -> IResult<&'a [u8], O>,
{
    move |input: &[u8]| {
        let (input, _) = multispace0(input)?;
        let (input, _) = tag(keyword)(input)?;
        let (input, _) = space1(input)?;
        let (input, p) = parameters(input)?;
        let (input, _) = space0(input)?;
        let (input, _) = opt(line_ending)(input)?;

        Ok((input, p))
    }
}

pub fn signed_xy(input: &[u8]) -> IResult<&[u8], (i32, i32)> {
    separated_pair(parse_to_i32, space1, parse_to_i32)(input)
}

pub fn unsigned_xy(input: &[u8]) -> IResult<&[u8], (u32, u32)> {
    separated_pair(parse_to_u32, space1, parse_to_u32)(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EMPTY: &[u8] = &[];

    #[test]
    fn it_takes_until_any_line_ending() {
        assert_eq!(
            take_until_line_ending(b"Unix line endings\n"),
            Ok((b"\n".as_ref(), b"Unix line endings".as_ref()))
        );

        assert_eq!(
            take_until_line_ending(b"Windows line endings\r\n"),
            Ok((b"\r\n".as_ref(), b"Windows line endings".as_ref()))
        );
    }

    #[test]
    fn it_parses_comments() {
        let comment_text = b"COMMENT test text\n";
        let out = comment(comment_text);

        assert_eq!(out, Ok((EMPTY, "test text".to_string())));

        // EMPTY comments
        assert_eq!(comment(b"COMMENT\n"), Ok((EMPTY, "".to_string())));
    }
}

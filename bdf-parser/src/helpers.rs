use nom::{
    bytes::complete::{tag, take_until, take_while},
    character::complete::{digit1, line_ending, multispace0, one_of, space0, space1},
    combinator::{map_opt, opt, recognize},
    multi::many0,
    sequence::{delimited, preceded},
    IResult, ParseTo,
};

pub fn parse_to_i32(input: &str) -> IResult<&str, i32> {
    map_opt(recognize(preceded(opt(one_of("+-")), digit1)), |i: &str| {
        i.parse_to()
    })(input)
}

pub fn parse_to_u32(input: &str) -> IResult<&str, u32> {
    map_opt(recognize(digit1), |i: &str| i.parse_to())(input)
}

fn comment(input: &str) -> IResult<&str, String> {
    map_opt(
        delimited(
            tag("COMMENT"),
            opt(preceded(space1, take_until("\n"))),
            line_ending,
        ),
        |c: Option<&str>| c.map_or(Some(String::from("")), |c| c.parse_to()),
    )(input)
}

pub fn skip_comments<'a, F, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: FnMut(&'a str) -> IResult<&'a str, O>,
{
    delimited(many0(comment), inner, many0(comment))
}

pub fn numchars(input: &str) -> IResult<&str, u32> {
    preceded(
        space0,
        preceded(tag("CHARS"), preceded(space0, parse_to_u32)),
    )(input)
}

pub fn take_until_line_ending(input: &str) -> IResult<&str, &str> {
    take_while(|c| c != '\n' && c != '\r')(input)
}

pub fn statement<'a, O, F>(
    keyword: &'a str,
    mut parameters: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: FnMut(&'a str) -> IResult<&'a str, O>,
{
    move |input: &str| {
        let (input, _) = multispace0(input)?;
        let (input, _) = tag(keyword)(input)?;
        let (input, _) = space1(input)?;
        let (input, p) = parameters(input)?;
        let (input, _) = space0(input)?;
        let (input, _) = opt(line_ending)(input)?;

        Ok((input, p))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_takes_until_any_line_ending() {
        assert_eq!(
            take_until_line_ending("Unix line endings\n"),
            Ok(("\n".as_ref(), "Unix line endings".as_ref()))
        );

        assert_eq!(
            take_until_line_ending("Windows line endings\r\n"),
            Ok(("\r\n".as_ref(), "Windows line endings".as_ref()))
        );
    }

    #[test]
    fn it_parses_comments() {
        let comment_text = "COMMENT test text\n";
        let out = comment(comment_text);

        assert_eq!(out, Ok(("", "test text".to_string())));

        // EMPTY comments
        assert_eq!(comment("COMMENT\n"), Ok(("", "".to_string())));
    }
}

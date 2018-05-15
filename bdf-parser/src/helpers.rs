use nom::*;

named!(
    pub parse_to_i32<i32>,
    flat_map!(
        recognize!(preceded!(opt!(one_of!("+-")), digit)),
        parse_to!(i32)
    )
);

named!(
    pub parse_to_u32<u32>,
    flat_map!(recognize!(digit), parse_to!(u32))
);

named!(
    pub comment<String>,
    flat_map!(
        delimited!(
            alt!(tag!("COMMENT ") | tag!("COMMENT")),
            take_until!("\n"),
            line_ending
        ),
        parse_to!(String)
    )
);

named!(pub optional_comments<Vec<String>>, many0!(comment));

named!(pub numchars<u32>, ws!(preceded!(tag!("CHARS"), parse_to_u32)));

named!(pub take_until_line_ending, alt_complete!(take_until!("\r\n") | take_until!("\n")));

#[cfg(test)]
mod tests {
    use super::*;
    use nom::IResult;

    const EMPTY: &[u8] = &[];

    #[test]
    fn it_takes_until_any_line_ending() {
        assert_eq!(
            take_until_line_ending("Unix line endings\n".as_bytes()),
            IResult::Done("\n".as_bytes(), "Unix line endings".as_bytes())
        );

        assert_eq!(
            take_until_line_ending("Windows line endings\r\n".as_bytes()),
            IResult::Done("\r\n".as_bytes(), "Windows line endings".as_bytes())
        );
    }

    #[test]
    fn it_parses_comments() {
        let comment_text = "COMMENT test text\n";
        let out = comment(comment_text.as_bytes());

        assert_eq!(out, IResult::Done(EMPTY, "test text".to_string()));

        // EMPTY comments
        assert_eq!(
            comment("COMMENT\n".as_bytes()),
            IResult::Done(EMPTY, "".to_string())
        );
    }
}

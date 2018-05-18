use nom::*;

use nom::types::CompleteByteSlice;

named!(
    pub parse_to_i32<CompleteByteSlice, i32>,
    flat_map!(
        recognize!(preceded!(opt!(one_of!("+-")), digit)),
        parse_to!(i32)
    )
);

named!(
    pub parse_to_u32<CompleteByteSlice, u32>,
    flat_map!(recognize!(digit), parse_to!(u32))
);

named!(
    pub comment<CompleteByteSlice, String>,
    flat_map!(
        delimited!(
            alt!(tag!("COMMENT ") | tag!("COMMENT")),
            take_until!("\n"),
            line_ending
        ),
        parse_to!(String)
    )
);

named!(pub optional_comments<CompleteByteSlice, Vec<String>>, many0!(comment));

named!(pub numchars<CompleteByteSlice, u32>, ws!(preceded!(tag!("CHARS"), parse_to_u32)));

named!(pub take_until_line_ending<CompleteByteSlice, CompleteByteSlice>, alt_complete!(take_until!("\r\n") | take_until!("\n")));

#[cfg(test)]
mod tests {
    use super::*;

    const EMPTY: CompleteByteSlice = CompleteByteSlice(b"");

    #[test]
    fn it_takes_until_any_line_ending() {
        assert_eq!(
            take_until_line_ending(CompleteByteSlice(b"Unix line endings\n")),
            Ok((
                CompleteByteSlice(b"\n"),
                CompleteByteSlice(b"Unix line endings")
            ))
        );

        assert_eq!(
            take_until_line_ending(CompleteByteSlice(b"Windows line endings\r\n")),
            Ok((
                CompleteByteSlice(b"\r\n"),
                CompleteByteSlice(b"Windows line endings")
            ))
        );
    }

    #[test]
    fn it_parses_comments() {
        let comment_text = b"COMMENT test text\n";
        let out = comment(CompleteByteSlice(comment_text));

        assert_eq!(out, Ok((EMPTY, "test text".to_string())));

        // EMPTY comments
        assert_eq!(
            comment(CompleteByteSlice(b"COMMENT\n")),
            Ok((EMPTY, "".to_string()))
        );
    }
}

use nom::types::CompleteByteSlice;

#[derive(Debug, Clone, PartialEq)]
pub struct Properties;

named!(
    pub properties<CompleteByteSlice, Properties>,
    map!(
        ws!(delimited!(
            tag!("STARTPROPERTIES"),
            take_until!("ENDPROPERTIES"),
            tag!("ENDPROPERTIES")
        )),
        |_| Properties
    )
);

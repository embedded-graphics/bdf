#[derive(Debug, Clone, PartialEq)]
pub struct Properties;

named!(
    pub properties<Properties>,
    map!(
        ws!(delimited!(
            tag!("STARTPROPERTIES"),
            take_until!("ENDPROPERTIES"),
            tag!("ENDPROPERTIES")
        )),
        |_| Properties
    )
);

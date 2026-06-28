use eg_bdf::BdfFont;

/// Serializes a u8 vector from a BDF font
///
/// This vector takes the format described in [eg-bdf::SerializedBdfFont]
pub fn serialize(font: BdfFont) -> anyhow::Result<Vec<u8>> {
    let mut data: Vec<u8> = Vec::new();

    macro_rules! append_be_data {
        ($e:expr, $t:ty) => {
            data.extend_from_slice(&(<$t>::try_from($e)?).to_be_bytes())
        };
    }

    // Header
    append_be_data!(font.ascent, u16);
    append_be_data!(font.descent, u16);
    append_be_data!(font.replacement_character, u32);
    append_be_data!(font.glyphs.len(), u32);

    // Character table
    for glyph in font.glyphs {
        append_be_data!(glyph.character, u32);
        append_be_data!(glyph.bounding_box.top_left.x, i16);
        append_be_data!(glyph.bounding_box.top_left.y, i16);
        append_be_data!(glyph.bounding_box.size.width, u16);
        append_be_data!(glyph.bounding_box.size.height, u16);
        append_be_data!(glyph.device_width, u8);
        append_be_data!(glyph.start_index, u32);
    }

    // Data
    data.extend_from_slice(font.data);

    Ok(data)
}

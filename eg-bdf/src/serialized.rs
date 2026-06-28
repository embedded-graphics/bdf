use crate::{DisplayBdfGlyph, ProportionalFont, ProportionalTextStyle};
use embedded_graphics::{prelude::*, primitives::Rectangle};

const fn get_be_i16(data: &[u8], idx: usize) -> i16 {
    i16::from_be_bytes([data[idx], data[idx + 1]])
}

const fn get_be_u16(data: &[u8], idx: usize) -> u16 {
    u16::from_be_bytes([data[idx], data[idx + 1]])
}

const fn get_be_u32(data: &[u8], idx: usize) -> u32 {
    u32::from_be_bytes([data[idx], data[idx + 1], data[idx + 2], data[idx + 3]])
}

/// Serialized BDF font.
///
/// # Binary Format
///
/// All integers are stored in big endian byte order.
///
/// * Header (12 Bytes):
///   * Ascent  (pixels, `u16`)
///   * Descent (pixels, `u16`)
///   * Replacement Character (index into character table, `u32`)
///   * Glyph Table Length (number of entries, `u32`)
///
/// * Glyph Table (17 Bytes Per Entry):
///   * corresponding codepoint (`u32`)
///   * `top_left.x` (`i16`)
///   * `top_left.y` (`i16`)
///   * `size.width` (`u16`)
///   * `size.height` (`u16`)
///   * `device_width` (pixels, `u8`)
///   * data index  (bytes from start of data, `u32`)
///
/// The bitmap data for all glyphs is stored after the header and glyph table. Each glyph bitmap
/// starts at a byte boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SerializedBdfFont<'a> {
    /// The raw u8 data of the serialized font
    data: &'a [u8],
}

impl<'a> SerializedBdfFont<'a> {
    // Structure Sizes
    const HEADER_SIZE: usize = 12;
    const GLYPH_TABLE_ENTRY_SIZE: usize = 17;

    // Offsets for the header
    const ASCENT_OFFSET: usize = 0;
    const DESCENT_OFFSET: usize = 2;
    const REPLACEMENT_OFFSET: usize = 4;
    const GLYPH_COUNT_OFFSET: usize = 8;

    // Offsets for the glyph table entries
    const CODEPOINT_OFFSET: usize = 0;
    const TOPLEFTX_OFFSET: usize = 4;
    const TOPLEFTY_OFFSET: usize = 6;
    const SIZEX_OFFSET: usize = 8;
    const SIZEY_OFFSET: usize = 10;
    const KERN_OFFSET: usize = 12;
    const IDX_OFFSET: usize = 13;

    const fn glyph_data(
        &self,
        index: usize,
    ) -> Option<&[u8; SerializedBdfFont::GLYPH_TABLE_ENTRY_SIZE]> {
        let (_header, data) = self
            .data
            .split_at(Self::HEADER_SIZE + (index * Self::GLYPH_TABLE_ENTRY_SIZE));
        data.first_chunk()
    }

    /// Verifies data in a way that prevents panics and returns a serialized font if the data is valid
    pub const fn new(data: &'a [u8]) -> Result<Self, &'static str> {
        // No header
        if data.len() < Self::HEADER_SIZE {
            return Err("No header");
        }

        // Safe to construct a font and index its header
        let font = Self { data };

        // Glyph table too small
        if data.len() < font.header_size() {
            return Err("No metadata");
        }

        // Data is okay
        Ok(font)
    }

    /// Returns the number of glyphs.
    pub const fn glyph_count(self) -> usize {
        get_be_u32(self.data, Self::GLYPH_COUNT_OFFSET) as usize
    }

    /// Returns the offset of the data block
    const fn header_size(self) -> usize {
        Self::HEADER_SIZE + self.glyph_count() * Self::GLYPH_TABLE_ENTRY_SIZE
    }

    /// Gets the glyph for the given index.
    ///
    /// Returns [`None`] if the index is out of bounds.
    pub fn lookup_by_index(self, index: usize) -> Option<DisplayBdfGlyph<'a>> {
        let glyph = self.glyph_data(index)?;
        let corresponding_character = char::from_u32(get_be_u32(glyph, Self::CODEPOINT_OFFSET))?;
        let top_left_x = get_be_i16(glyph, Self::TOPLEFTX_OFFSET);
        let top_left_y = get_be_i16(glyph, Self::TOPLEFTY_OFFSET);
        let width = get_be_u16(glyph, Self::SIZEX_OFFSET);
        let height = get_be_u16(glyph, Self::SIZEY_OFFSET);
        let kerning = glyph[Self::KERN_OFFSET];

        let data_index = get_be_u32(glyph, Self::IDX_OFFSET);
        let bitmap_data = self
            .data
            .get((self.header_size() + data_index as usize)..)?;

        Some(DisplayBdfGlyph {
            character: corresponding_character,
            bounding_box: Rectangle {
                top_left: Point {
                    x: i32::from(top_left_x),
                    y: i32::from(top_left_y),
                },
                size: Size {
                    width: u32::from(width),
                    height: u32::from(height),
                },
            },
            device_width: u32::from(kerning),
            bitmap_data: bitmap_data,
        })
    }
}
impl<'a> ProportionalFont<'a> for SerializedBdfFont<'a> {
    fn metrics(&self) -> crate::proportional::Metrics {
        crate::proportional::Metrics {
            ascent: u32::from(get_be_u16(self.data, Self::ASCENT_OFFSET)),
            descent: u32::from(get_be_u16(self.data, Self::DESCENT_OFFSET)),
            line_height: u32::from(get_be_u16(self.data, Self::ASCENT_OFFSET))
                + u32::from(get_be_u16(self.data, Self::DESCENT_OFFSET)),
        }
    }

    fn replacement_glyph(&self) -> DisplayBdfGlyph<'_> {
        let rpos = get_be_u32(self.data, Self::REPLACEMENT_OFFSET);
        self.lookup_by_index(rpos as usize)
            .expect("Replacement character isn't valid")
    }

    fn lookup(&self, c: char) -> Option<DisplayBdfGlyph<'_>> {
        // TODO, make this a binary search
        for i in 0..self.glyph_count() {
            let Some(glyph) = self.lookup_by_index(i) else {
                // skip invalid glyphs
                continue;
            };

            if glyph.character == c {
                return Some(glyph);
            }
        }

        None
    }
}

/// Stylized serialized BDF text
pub type SerializedBdfTextStyle<'a, C> = ProportionalTextStyle<'a, SerializedBdfFont<'a>, C>;

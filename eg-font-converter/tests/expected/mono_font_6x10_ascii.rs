pub const MONO_FONT_6X10_ASCII: ::embedded_graphics::mono_font::MonoFont = ::embedded_graphics::mono_font::MonoFont {
    image: ::embedded_graphics::image::ImageRaw::new(
        include_bytes!("mono_font_6x10_ascii.data"),
        96u32,
    ),
    glyph_mapping: &::embedded_graphics::mono_font::mapping::ASCII,
    character_size: ::embedded_graphics::geometry::Size::new(6u32, 10u32),
    character_spacing: 0u32,
    baseline: 7u32,
    underline: ::embedded_graphics::mono_font::DecorationDimensions::new(9u32, 1u32),
    strikethrough: ::embedded_graphics::mono_font::DecorationDimensions::new(5u32, 1u32),
};

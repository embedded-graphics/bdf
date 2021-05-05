pub const MONO_FONT_10X20_ISO8859_15: ::embedded_graphics::mono_font::MonoFont = ::embedded_graphics::mono_font::MonoFont {
    image: ::embedded_graphics::image::ImageRaw::new(
        include_bytes!("mono_font_10x20_iso8859_15.data"),
        160u32,
    ),
    glyph_mapping: &::embedded_graphics::mono_font::mapping::ISO_8859_15,
    character_size: ::embedded_graphics::geometry::Size::new(10u32, 20u32),
    character_spacing: 0u32,
    baseline: 15u32,
    underline: ::embedded_graphics::mono_font::DecorationDimensions::new(17u32, 1u32),
    strikethrough: ::embedded_graphics::mono_font::DecorationDimensions::new(10u32, 1u32),
};

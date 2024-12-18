pub const EG_BDF_AZ: ::eg_bdf::BdfFont = {
    const fn rect(
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> ::embedded_graphics::primitives::Rectangle {
        ::embedded_graphics::primitives::Rectangle::new(
            ::embedded_graphics::geometry::Point::new(x, y),
            ::embedded_graphics::geometry::Size::new(width, height),
        )
    }
    ::eg_bdf::BdfFont {
        data: include_bytes!("eg_bdf_az.data"),
        replacement_character: 0usize,
        ascent: 8u32,
        descent: 2u32,
        glyphs: &[
            ::eg_bdf::BdfGlyph {
                character: 'a',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 0usize,
            },
            ::eg_bdf::BdfGlyph {
                character: 'b',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 60usize,
            },
            ::eg_bdf::BdfGlyph {
                character: 'c',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 120usize,
            },
            ::eg_bdf::BdfGlyph {
                character: 'd',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 180usize,
            },
            ::eg_bdf::BdfGlyph {
                character: 'e',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 240usize,
            },
            ::eg_bdf::BdfGlyph {
                character: 'f',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 300usize,
            },
            ::eg_bdf::BdfGlyph {
                character: 'g',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 360usize,
            },
            ::eg_bdf::BdfGlyph {
                character: 'h',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 420usize,
            },
            ::eg_bdf::BdfGlyph {
                character: 'i',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 480usize,
            },
            ::eg_bdf::BdfGlyph {
                character: 'j',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 540usize,
            },
            ::eg_bdf::BdfGlyph {
                character: 'k',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 600usize,
            },
            ::eg_bdf::BdfGlyph {
                character: 'l',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 660usize,
            },
            ::eg_bdf::BdfGlyph {
                character: 'm',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 720usize,
            },
            ::eg_bdf::BdfGlyph {
                character: 'n',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 780usize,
            },
            ::eg_bdf::BdfGlyph {
                character: 'o',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 840usize,
            },
            ::eg_bdf::BdfGlyph {
                character: 'p',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 900usize,
            },
            ::eg_bdf::BdfGlyph {
                character: 'q',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 960usize,
            },
            ::eg_bdf::BdfGlyph {
                character: 'r',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 1020usize,
            },
            ::eg_bdf::BdfGlyph {
                character: 's',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 1080usize,
            },
            ::eg_bdf::BdfGlyph {
                character: 't',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 1140usize,
            },
            ::eg_bdf::BdfGlyph {
                character: 'u',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 1200usize,
            },
            ::eg_bdf::BdfGlyph {
                character: 'v',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 1260usize,
            },
            ::eg_bdf::BdfGlyph {
                character: 'w',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 1320usize,
            },
            ::eg_bdf::BdfGlyph {
                character: 'x',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 1380usize,
            },
            ::eg_bdf::BdfGlyph {
                character: 'y',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 1440usize,
            },
            ::eg_bdf::BdfGlyph {
                character: 'z',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 1500usize,
            },
        ],
    }
};

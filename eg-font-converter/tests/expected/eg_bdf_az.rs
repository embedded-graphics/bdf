const EG_BDF_AZ: ::eg_bdf::BdfFont = {
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
            BdfGlyph {
                character: 'a',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 0usize,
            },
            BdfGlyph {
                character: 'b',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 60usize,
            },
            BdfGlyph {
                character: 'c',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 120usize,
            },
            BdfGlyph {
                character: 'd',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 180usize,
            },
            BdfGlyph {
                character: 'e',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 240usize,
            },
            BdfGlyph {
                character: 'f',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 300usize,
            },
            BdfGlyph {
                character: 'g',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 360usize,
            },
            BdfGlyph {
                character: 'h',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 420usize,
            },
            BdfGlyph {
                character: 'i',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 480usize,
            },
            BdfGlyph {
                character: 'j',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 540usize,
            },
            BdfGlyph {
                character: 'k',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 600usize,
            },
            BdfGlyph {
                character: 'l',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 660usize,
            },
            BdfGlyph {
                character: 'm',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 720usize,
            },
            BdfGlyph {
                character: 'n',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 780usize,
            },
            BdfGlyph {
                character: 'o',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 840usize,
            },
            BdfGlyph {
                character: 'p',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 900usize,
            },
            BdfGlyph {
                character: 'q',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 960usize,
            },
            BdfGlyph {
                character: 'r',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 1020usize,
            },
            BdfGlyph {
                character: 's',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 1080usize,
            },
            BdfGlyph {
                character: 't',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 1140usize,
            },
            BdfGlyph {
                character: 'u',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 1200usize,
            },
            BdfGlyph {
                character: 'v',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 1260usize,
            },
            BdfGlyph {
                character: 'w',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 1320usize,
            },
            BdfGlyph {
                character: 'x',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 1380usize,
            },
            BdfGlyph {
                character: 'y',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 1440usize,
            },
            BdfGlyph {
                character: 'z',
                bounding_box: rect(0i32, -7i32, 6u32, 10u32),
                device_width: 6u32,
                start_index: 1500usize,
            },
        ],
    }
};

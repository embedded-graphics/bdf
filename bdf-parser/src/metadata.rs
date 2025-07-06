use crate::{
    parser::{Lines, ParserError},
    BoundingBox, Coord, Properties,
};

/// Metrics set.
///
/// The metrics set specifies for which writing directions the font includes metrics.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum MetricsSet {
    /// Horizontal writing direction.
    ///
    /// `METRICSSET 0`
    #[default]
    Horizontal,
    /// Vertical writing direction.
    ///
    /// `METRICSSET 1`
    Vertical,
    /// Both writing directions.
    ///
    /// `METRICSSET 2`
    Both,
}

/// BDF file metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata {
    /// Font name.
    pub name: String,

    /// Point size.
    pub point_size: i32,

    /// X and Y resolution in DPI.
    pub resolution: Coord,

    /// Font bounding box.
    pub bounding_box: BoundingBox,

    /// Metrics set.
    pub metrics_set: MetricsSet,

    /// Properties.
    pub properties: Properties,
}

impl Metadata {
    pub(crate) fn parse<'a>(lines: &mut Lines<'a>) -> Result<Self, ParserError> {
        let mut name = None;
        let mut font_bounding_box = None;
        let mut point_size = None;
        let mut resolution = Coord::default();
        let mut metrics_set = MetricsSet::default();
        let mut properties = None;

        while let Some(line) = lines.next() {
            match line.keyword {
                "FONT" => {
                    name = Some(line.parameters.to_string());
                }
                "FONTBOUNDINGBOX" => {
                    font_bounding_box = Some(BoundingBox::parse(&line).ok_or_else(|| {
                        ParserError::with_line("invalid \"FONTBOUNDINGBOX\"", &line)
                    })?);
                }
                "SIZE" => {
                    let [point, x, y] = line
                        .parse_integer_parameters()
                        .ok_or_else(|| ParserError::with_line("invalid \"SIZE\"", &line))?;
                    point_size = Some(point);
                    resolution.x = x;
                    resolution.y = y;
                }
                "METRICSSET" => {
                    let [index] = line
                        .parse_integer_parameters()
                        .filter(|[index]| (0..=2).contains(index))
                        .ok_or_else(|| ParserError::with_line("invalid \"METRICSSET\"", &line))?;

                    metrics_set = match index {
                        0 => MetricsSet::Horizontal,
                        1 => MetricsSet::Vertical,
                        2 => MetricsSet::Both,
                        _ => unreachable!(),
                    }
                }
                "STARTPROPERTIES" => {
                    lines.backtrack(line);
                    properties = Some(Properties::parse(lines)?);
                }
                "CHARS" | "STARTCHAR" => {
                    lines.backtrack(line);
                    break;
                }
                _ => {
                    return Err(ParserError::with_line(
                        &format!("unknown keyword in metadata: \"{}\"", line.keyword),
                        &line,
                    ))
                }
            }
        }

        if name.is_none() {
            return Err(ParserError::new("missing \"FONT\""));
        }
        if font_bounding_box.is_none() {
            return Err(ParserError::new("missing \"FONTBOUNDINGBOX\""));
        }
        if point_size.is_none() {
            return Err(ParserError::new("missing \"SIZE\""));
        }

        Ok(Metadata {
            name: name.unwrap(),
            point_size: point_size.unwrap(),
            resolution,
            bounding_box: font_bounding_box.unwrap(),
            metrics_set,
            properties: properties.unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;
    use crate::{tests::assert_parser_error, Font};

    #[test]
    fn complete_metadata() {
        const FONT: &str = indoc! {r#"
            STARTFONT 2.1
            FONT "test font"
            FONTBOUNDINGBOX 0 1 2 3
            SIZE 1 2 3
            COMMENT "comment"
            CHARS 0
            ENDFONT
        "#};

        Font::parse(FONT).unwrap();
    }

    #[test]
    fn missing_name() {
        const FONT: &str = indoc! {r#"
            STARTFONT 2.1
            FONTBOUNDINGBOX 0 1 2 3
            SIZE 1 2 3
            CHARS 0
            ENDFONT
        "#};

        assert_parser_error(FONT, "missing \"FONT\"", None);
    }

    #[test]
    fn missing_fontboundingbox() {
        const FONT: &str = indoc! {r#"
            STARTFONT 2.1
            FONT "test font"
            SIZE 1 2 3
            CHARS 0
            ENDFONT
        "#};

        assert_parser_error(FONT, "missing \"FONTBOUNDINGBOX\"", None);
    }

    #[test]
    fn missing_size() {
        const FONT: &str = indoc! {r#"
            STARTFONT 2.1
            FONT "test font"
            FONTBOUNDINGBOX 0 1 2 3
            CHARS 0
            ENDFONT
        "#};

        assert_parser_error(FONT, "missing \"SIZE\"", None);
    }

    #[test]
    fn metrics_set() {
        const FONT: &str = indoc! {r#"
            STARTFONT 2.1
            FONT "test font"
            FONTBOUNDINGBOX 0 1 2 3
            SIZE 1 2 3
            COMMENT "comment"
            METRICSSET 2
            CHARS 0
            ENDFONT
        "#};

        let font = Font::parse(FONT).unwrap();
        assert_eq!(font.metadata.metrics_set, MetricsSet::Both);
    }
}

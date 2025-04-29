//! Font analysis module.

use std::collections::HashMap;
use thiserror::Error;
use ttf_parser::{Face, GlyphId, OutlineBuilder, Rect};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Font Face Parse Error: {0}")]
    FaceParse(#[from] ttf_parser::FaceParsingError),
}

#[derive(Debug, Copy, Clone)]
pub struct BoundingBox {
    pub x_min: i16,
    pub y_min: i16,
    pub x_max: i16,
    pub y_max: i16,
}

/// Glyph structure
#[derive(Debug)]
pub struct Glyph {
    /// Path
    pub path: String,
    /// Bounding box path
    pub bbox_path: String,
    /// Bounding box
    pub bbox: BoundingBox,
}

/// Font analysis structure
#[derive(Debug)]
pub struct FontAnalysis {
    /// Descender
    pub descender: i16,
    /// Ascender
    pub ascender: i16,
    /// Maximum height of the font
    pub y_max: i16,
    /// Average glyph width
    pub glyph_width_avg: f64,
    /// HashMap of glyphs
    pub glyphs: HashMap<char, Glyph>,
}

struct Builder<'a>(&'a mut String);

impl Builder<'_> {
    fn finish(&mut self) {
        if !self.0.is_empty() {
            self.0.pop(); // remove trailing space
        }
    }
}

impl OutlineBuilder for Builder<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        use std::fmt::Write;
        write!(self.0, "M {} {} ", x, y).unwrap()
    }

    fn line_to(&mut self, x: f32, y: f32) {
        use std::fmt::Write;
        write!(self.0, "L {} {} ", x, y).unwrap()
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        use std::fmt::Write;
        write!(self.0, "Q {} {} {} {} ", x1, y1, x, y).unwrap()
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        use std::fmt::Write;
        write!(self.0, "C {} {} {} {} {} {} ", x1, y1, x2, y2, x, y).unwrap()
    }

    fn close(&mut self) {
        self.0.push_str("Z ")
    }
}
fn generate_path(face: &Face, glyph_id: GlyphId) -> (String, BoundingBox) {
    let mut path_buf = String::new();
    let mut builder = Builder(&mut path_buf);
    let bbox = match face.outline_glyph(glyph_id, &mut builder) {
        Some(v) => v,
        None => Rect {
            x_min: 0,
            y_min: 0,
            x_max: 0,
            y_max: 0,
        },
    };
    builder.finish();
    let custom_bbox = BoundingBox {
        x_min: bbox.x_min,
        y_min: bbox.y_min,
        x_max: bbox.x_max,
        y_max: bbox.y_max,
    };
    (path_buf, custom_bbox)
}

impl FontAnalysis {
    /// Create a new font analysis from a TTF file
    pub fn analyze(font: Vec<u8>) -> Result<Self, Error> {
        let face = Face::parse(&font, 0)?;
        let units_per_em = face.units_per_em();
        println!("Units per EM: {:?}", units_per_em);
        println!("Ascender: {}", face.ascender());
        println!("Descender: {}", face.descender());
        println!("Line gap: {}", face.line_gap());
        println!("Global bbox: {:?}", face.global_bounding_box());
        println!("Number of glyphs: {}", face.number_of_glyphs());
        println!("Underline: {:?}", face.underline_metrics());
        println!("Height: {}", face.height());
        println!("X height: {:?}", face.x_height());
        println!("Weight: {:?}", face.weight());
        println!("Width: {:?}", face.width());
        println!("Variable: {:?}", face.is_variable());
        let mut glyphs = HashMap::new();
        let mut y_max = i16::MIN;
        let mut glyphs_count = 0;
        let mut glyph_width_sum = 0;
        for c in ('A'..='Z').chain(vec!['-']) {
            if let Some(glyph_id) = face.glyph_index(c) {
                let (path, bb) = generate_path(&face, glyph_id);
                println!("Glyph {:?} bounding box: {:?}", c, bb);
                if bb.y_max > y_max {
                    y_max = bb.y_max;
                }
                glyphs_count += 1;
                glyph_width_sum += bb.x_max - bb.x_min;
                let bbox_path = format!(
                    "M {} {} L {} {} L {} {} L {} {} Z",
                    bb.x_min, bb.y_min, bb.x_max, bb.y_min, bb.x_max, bb.y_max, bb.x_min, bb.y_max
                );
                let glyph = Glyph {
                    path,
                    bbox_path,
                    bbox: bb,
                };
                glyphs.insert(c, glyph);
            }
        }
        let flower = '\u{2698}';
        if let Some(glyph_id) = face.glyph_index(flower) {
            let (path, bb) = generate_path(&face, glyph_id);
            println!("Glyph {:?} bounding box: {:?}", ' ', bb);
            let bbox_path = format!(
                "M {} {} L {} {} L {} {} L {} {} Z",
                bb.x_min, bb.y_min, bb.x_max, bb.y_min, bb.x_max, bb.y_max, bb.x_min, bb.y_max
            );
            let glyph = Glyph {
                path,
                bbox_path,
                bbox: bb,
            };
            glyphs.insert(flower, glyph);
        }
        let descender = face.descender();
        let ascender = face.ascender();
        Ok(Self {
            descender,
            ascender,
            y_max,
            glyph_width_avg: glyph_width_sum as f64 / glyphs_count as f64,
            glyphs,
        })
    }
}

/// Analyze a font file
pub fn analyze_font(font: Vec<u8>) -> Result<FontAnalysis, Error> {
    let analysis = FontAnalysis::analyze(font)?;
    Ok(analysis)
}

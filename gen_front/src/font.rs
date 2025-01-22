//! Font analysis module.

use std::collections::HashMap;
use thiserror::Error;
use ttf_parser::{Face, Rect};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Font Face Parse Error: {0}")]
    FaceParse(#[from] ttf_parser::FaceParsingError),
}

/// Glyph structure
#[derive(Debug)]
pub struct Glyph {
    /// Bounding box
    bounding_box: Rect,
}

/// Font analysis structure
#[derive(Debug)]
pub struct FontAnalysis {
    /// General bounding box
    pub bounding_box: Rect,
    /// HashMap of glyphs
    pub glyphs: HashMap<char, Glyph>,
}

impl FontAnalysis {
    /// Create a new font analysis from a TTF file
    pub fn analyze(font: Vec<u8>) -> Result<Self, Error> {
        let face = Face::parse(&font, 0)?;
        println!("Units per EM: {:?}", face.units_per_em());
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
        let mut bounding_box = Rect {
            x_min: i16::MAX,
            y_min: i16::MAX,
            x_max: i16::MIN,
            y_max: i16::MIN,
        };
        for c in ('A'..='Z').chain(vec!['-']) {
            if let Some(glyph_id) = face.glyph_index(c) {
                let bb = face.glyph_bounding_box(glyph_id).unwrap();
                println!("Glyph {:?} bounding box: {:?}", c, bb);
                if bb.x_min < bounding_box.x_min {
                    bounding_box.x_min = bb.x_min;
                }
                if bb.y_min < bounding_box.y_min {
                    bounding_box.y_min = bb.y_min;
                }
                if bb.x_max > bounding_box.x_max {
                    bounding_box.x_max = bb.x_max;
                }
                if bb.y_max > bounding_box.y_max {
                    bounding_box.y_max = bb.y_max;
                }
                let glyph = Glyph { bounding_box: bb };
                glyphs.insert(c, glyph);
            }
        }
        let flower = '\u{2698}';
        if let Some(glyph_id) = face.glyph_index(flower) {
            let bb = face.glyph_bounding_box(glyph_id).unwrap();
            println!("Glyph {:?} bounding box: {:?}", ' ', bb);
            let glyph = Glyph { bounding_box: bb };
            glyphs.insert(flower, glyph);
        }
        //let units_per_em = face.units_per_em();
        //let cell_size = face.height() as f64 * FONT_SIZE / units_per_em as f64;
        Ok(Self {
            bounding_box,
            glyphs,
        })
    }
}

/// Analyze a font file
pub fn analyze_font(font: Vec<u8>) -> Result<FontAnalysis, Error> {
    let analysis = FontAnalysis::analyze(font)?;
    Ok(analysis)
}

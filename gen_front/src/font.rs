//! Font analysis module.

use thiserror::Error;
use ttf_parser::Face;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Font Face Parse Error: {0}")]
    FaceParse(#[from] ttf_parser::FaceParsingError),
}

/// Font analysis structure
#[derive(Debug)]
pub struct FontAnalysis {}

impl FontAnalysis {
    /// Create a new font analysis from a TTF file
    pub fn new(font: Vec<u8>) -> Result<Self, Error> {
        let _face = Face::parse(&font, 0)?;
        Ok(Self {})
    }
}

/// Analyze a font file
pub fn analyze_font(font: Vec<u8>) -> Result<FontAnalysis, Error> {
    let font = FontAnalysis::new(font)?;
    Ok(font)
}

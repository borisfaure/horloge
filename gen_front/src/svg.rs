//! Generate the SVG file

use quick_xml::events::{BytesDecl, Event};
use quick_xml::Writer;
use std::fs::File;
use std::io::BufWriter;
use std::io::Result as IoResult;
use std::path::PathBuf;

use crate::font::FontAnalysis;

// Letter N: 31.45mm width, 51mm height
const LED_SIZE: f64 = 5f64;
const LED_SPACING: f64 = 12f64 + LED_SIZE;
const FONT_SIZE: f64 = 128f64;

const GRID_WIDTH: usize = 11;
const GRID_HEIGHT: usize = 10;
const MARGIN: f64 = 20.0;
#[cfg(feature = "french")]
const GRID: [[char; GRID_WIDTH]; GRID_HEIGHT] = [
    ['I', 'L', 'B', 'E', 'S', 'T', 'W', 'C', 'I', 'N', 'Q'],
    ['D', 'E', 'U', 'X', 'S', 'E', 'P', 'T', 'U', 'N', 'E'],
    ['Q', 'U', 'A', 'T', 'R', 'E', 'T', 'R', 'O', 'I', 'S'],
    ['N', 'E', 'U', 'F', 'S', 'I', 'X', 'H', 'U', 'I', 'T'],
    ['M', 'I', 'D', 'I', 'X', 'M', 'I', 'N', 'U', 'I', 'T'],
    ['O', 'N', 'Z', 'E', 'J', 'H', 'E', 'U', 'R', 'E', 'S'],
    ['L', 'M', 'O', 'I', 'N', 'S', 'K', 'C', 'I', 'N', 'Q'],
    ['E', 'T', 'Y', 'D', 'I', 'X', 'D', 'E', 'M', 'I', 'E'],
    ['M', 'V', 'I', 'N', 'G', 'T', '-', 'C', 'I', 'N', 'Q'],
    ['D', 'L', 'E', 'R', 'Q', 'U', 'A', 'R', 'T', 'B', 'F'],
];
#[cfg(feature = "english")]
const GRID: [[char; GRID_WIDTH]; GRID_HEIGHT] = [
    ['Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z'],
    ['Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z'],
    ['Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z'],
    ['Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z'],
    ['Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z'],
    ['Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z'],
    ['Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z'],
    ['Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z'],
    ['Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z'],
    ['Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z', 'Z'],
];

#[derive(Debug)]
struct Sizes {
    document_width: f64,
    document_height: f64,
}

impl Sizes {
    fn compute(font: &FontAnalysis) -> Sizes {
        // We need to compute the best font size so that the grid is square.
        // We know the horizontal spacing between the LEDs, the size of the
        // LEDs, and the ratio of the letters
        // The relationship between horizontal (`s`) and vertical (`v`) spacing is:
        // s = k * v, where k = h / l (letter height-to-width ratio).
        //
        // Variables:
        // - l: Width of a letter.
        // - h: Height of a letter, h = k * l.
        // - d: Spacing between the centers of consecutive LEDs.
        // - k: Ratio of letter height to width, k = font.y_max / font.glyph_width_avg.
        // - H: Height of the grid in terms of number of letters.
        // - W: Width of the grid in terms of number of letters.

        let d: f64 = LED_SPACING;
        println!("LED spacing: {}", d);

        let k: f64 = font.y_max as f64 / font.glyph_width_avg;
        println!("Letter ratio (height/width): {}", k);

        const H: f64 = GRID_HEIGHT as f64; // Number of rows in the grid
        const W: f64 = GRID_WIDTH as f64; // Number of columns in the grid

        // Ensure variables are valid to avoid undefined behavior
        assert!(H > 1.0, "Grid height must be greater than 1.");
        assert!(W > 1.0, "Grid width must be greater than 1.");
        assert!(k > 0.0, "Letter ratio (k) must be greater than 0.");

        // Derivation of the formula for `l`:
        // ----------------------------------------------------------
        // 1. Horizontal space (`s`): s = d - l
        // 2. Vertical space (`v`): v = s / k = (d - l) / k
        //
        // 3. Width of the square (`SqW`), width of all the letters + the
        //    space between them:
        //    SqW = W * l + (W - 1) * s
        //    Substituting s = d - l:
        //    SqW = W * l + (W - 1) * (d - l)
        //    Expanding:
        //    SqW = W * l + W * d - W * l - d + l
        //    Simplifying by canceling out `W * l`:
        //    SqW = W * d - d + l
        //
        // 4. Height of the square (`SqH`), height of all the letters + the
        //    vertical space between them:
        //    SqH = H * h + (H - 1) * v
        //    Substituting h = k * l and v = (d - l) / k:
        //    SqH = H * (k * l) + (H - 1) * ((d - l) / k)
        //    Expanding (d - l) / k:
        //    SqH = H * k * l + (H - 1) * d / k - (H - 1) * l / k
        //    Grouping terms with `l`:
        //    SqH = l * (H * k - (H - 1) / k) + (H - 1) * d / k
        //
        // 5. Squareness condition: SqW = SqH
        //    Equating the two expressions:
        //    W * d - d + l = l * (H * k - (H - 1) / k) + (H - 1) * d / k
        //
        // 6. Rearrange to isolate `l`:
        //    W * d - d - (H - 1) * d / k = l * (H * k - (H - 1) / k - 1)
        //    Simplifying by grouping terms with `d`:
        //    d * (W - 1 - (H - 1) / k) = l * (H * k - (H - 1) / k - 1)
        //    Solving for `l`:
        //    l = d * (W - 1 - (H - 1) / k) / (H * k - (H - 1) / k - 1)
        // ----------------------------------------------------------

        // Using the derived formula:
        let numerator = d * (W - 1.0 - (H - 1.0) / k);
        let denominator = H * k - (H - 1.0) / k - 1.0;

        // Check for potential division by zero
        assert!(
            denominator != 0.0,
            "Denominator in the font size calculation must not be zero."
        );

        let l: f64 = numerator / denominator;

        // Output the computed letter width
        println!("Computed letter width (l): {}", l);

        let glyph_width = l;
        let hspace = d - glyph_width;
        let vspace = hspace / k;
        println!("Letter width: {}", glyph_width);
        println!("Letter height: {}", glyph_width * k);
        println!("Letter spacing: horizontal {}, vertical {}", hspace, vspace);
        let sq_width = W * glyph_width + (W - 1f64) * hspace;
        let sq_height = H * glyph_width * k + (H - 1f64) * vspace;
        println!("Square width: {}", sq_width);
        println!("Square height: {}", sq_height);

        let document_width = sq_width + 2f64 * MARGIN;
        let document_height = sq_height + 2f64 * MARGIN;

        Sizes {
            document_width,
            document_height,
        }
    }
}

/// Write the grid to the SVG file
fn write_grid(
    writer: &mut Writer<BufWriter<File>>,
    font: FontAnalysis,
    scale: f64,
) -> IoResult<()> {
    for (y, row) in GRID.iter().enumerate().take(GRID_HEIGHT) {
        for (x, c) in row.iter().enumerate().take(GRID_WIDTH) {
            let x_str = (x as f64 * LED_SPACING + MARGIN).to_string();
            let y_str = (y as f64 * LED_SPACING + MARGIN).to_string();
            let glyph = font.glyphs.get(c).unwrap();
            let path = glyph.path.clone();
            let _bbox = glyph.bounding_box;
            //let bbox_w = (bbox.x_max as f64 - bbox.x_min as f64) * scale;
            //let dx = (cell_size - bbox_w) / 2.0;

            let transform = format!("matrix({} 0 0 {} {} {})", scale, -scale, x_str, y_str);
            let attrs = vec![
                ("d", path.as_str()),
                ("transform", transform.as_str()),
                ("stroke", "black"),
                ("stroke-width", "5"),
                ("fill", "none"),
            ];
            writer
                .create_element("path")
                .with_attributes(attrs.into_iter())
                .write_empty()?;
        }
    }
    Ok(())
}

/// Generate the SVG file
pub fn generate(file: &PathBuf, font: FontAnalysis) -> IoResult<()> {
    let mut writer = Writer::new_with_indent(BufWriter::new(File::create(file)?), b' ', 2);
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

    let units_per_em = font.units_per_em;
    let scale = FONT_SIZE / units_per_em as f64;
    println!("scale:{}", scale);
    let sizes = Sizes::compute(&font);
    println!(
        "Document size: {}x{}",
        sizes.document_width, sizes.document_height
    );
    let width_mm = format!("{}mm", sizes.document_width);
    let height_mm = format!("{}mm", sizes.document_height);
    let view_box = format!("0 0 {} {}", sizes.document_width, sizes.document_height);
    let svg_attrs = vec![
        ("width", width_mm.as_str()),
        ("height", height_mm.as_str()),
        ("viewBox", view_box.as_str()),
        ("version", "1.1"),
        ("id", "svg"),
        ("xmlns", "http://www.w3.org/2000/svg"),
        ("xmlns:svg", "http://www.w3.org/2000/svg"),
    ];
    writer
        .create_element("svg")
        .with_attributes(svg_attrs.into_iter())
        .write_inner_content(|writer| {
            writer
                .create_element("g")
                .with_attributes(vec![("id", "grid")].into_iter())
                .write_inner_content(|w| write_grid(w, font, scale))?;
            Ok(())
        })?;
    Ok(())
}

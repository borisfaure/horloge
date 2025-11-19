//! Generate the SVG file

use quick_xml::events::{BytesDecl, Event};
use quick_xml::Writer;
use std::fs::File;
use std::io::BufWriter;
use std::io::Result as IoResult;
use std::path::PathBuf;

#[cfg(feature = "fill")]
const FILL_COLOR: &str = "black";
//const FILL_COLOR: &str = "darkorange";

use crate::font::{FontAnalysis, FLOWER};

// Letter N: 31.45mm width, 51mm height
const LED_SIZE: f64 = 5f64;
const LED_SPACING: f64 = 12f64 + LED_SIZE;

const HOLE_DIAMETER: f64 = 3.3; // M4 according to
                                // https://www.laserboost.com/design-guide-for-threaded-and-counterbored-components/

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
    scale: f64,
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
        let glyph_height = glyph_width * k;
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

        let scale = glyph_height / font.y_max as f64;
        Sizes {
            scale,
            document_width,
            document_height,
        }
    }
}

/// Draw the LEDs in the grid
#[cfg(feature = "draw_leds")]
fn draw_leds(writer: &mut Writer<BufWriter<File>>, doc: &Sizes) -> IoResult<()> {
    // Compute the horizontal offset to center the LEDs
    let square_width = doc.document_width - 2.0 * MARGIN;
    let right_offset = (square_width - (GRID_WIDTH as f64 - 1.) * LED_SPACING - LED_SIZE) / 2.;
    // Compute the vertical spacing between the LEDs
    let square_height = doc.document_height - 2.0 * MARGIN;
    let vert_spacing = square_height / (GRID_HEIGHT as f64);

    let base_y = MARGIN + vert_spacing / 2. - LED_SIZE / 2.;
    let base_x = MARGIN + right_offset;
    let led_size = LED_SIZE.to_string();
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let x_str = (x as f64 * LED_SPACING + base_x).to_string();
            let y_str = (y as f64 * vert_spacing + base_y).to_string();
            let attrs = vec![
                ("x", x_str.as_str()),
                ("y", y_str.as_str()),
                ("width", led_size.as_str()),
                ("height", led_size.as_str()),
                ("fill", "red"),
            ];
            writer
                .create_element("rect")
                .with_attributes(attrs.into_iter())
                .write_empty()?;
        }
    }
    Ok(())
}

/// Draw the margins of the SVG file based on the document size and the
/// constant MARGIN
///
/// Draw 4 rectangles in blue that represent the margins of the document
#[cfg(feature = "draw_margins")]
fn draw_margins(writer: &mut Writer<BufWriter<File>>, doc: &Sizes) -> IoResult<()> {
    let width = doc.document_width.to_string();
    let margin = MARGIN.to_string();
    /* Top margin */
    let attrs = vec![
        ("x", "0"),
        ("y", "0"),
        ("width", width.as_str()),
        ("height", margin.as_str()),
        ("fill", "lightblue"),
    ];
    writer
        .create_element("rect")
        .with_attributes(attrs.into_iter())
        .write_empty()?;
    /* Bottom margin */
    let y = (doc.document_height - MARGIN).to_string();
    let attrs = vec![
        ("x", "0"),
        ("y", y.as_str()),
        ("width", width.as_str()),
        ("height", margin.as_str()),
        ("fill", "lightblue"),
    ];
    writer
        .create_element("rect")
        .with_attributes(attrs.into_iter())
        .write_empty()?;
    /* Left margin */
    let height = doc.document_height.to_string();
    let attrs = vec![
        ("x", "0"),
        ("y", "0"),
        ("width", margin.as_str()),
        ("height", height.as_str()),
        ("fill", "lightblue"),
    ];
    writer
        .create_element("rect")
        .with_attributes(attrs.into_iter())
        .write_empty()?;
    /* Right margin */
    let x = (doc.document_width - MARGIN).to_string();
    let attrs = vec![
        ("x", x.as_str()),
        ("y", "0"),
        ("width", margin.as_str()),
        ("height", height.as_str()),
        ("fill", "lightblue"),
    ];
    writer
        .create_element("rect")
        .with_attributes(attrs.into_iter())
        .write_empty()?;
    Ok(())
}

/// Draw a single minute glyph at position (x, y)
fn draw_minute(
    writer: &mut Writer<BufWriter<File>>,
    path: &str,
    scale: f64,
    x: f64,
    y: f64,
) -> IoResult<()> {
    // Position on top side, in the middlej
    // Position on the left side
    let transform = format!("matrix({} 0 0 {} {} {})", scale, -scale, x, y);
    let attrs = vec![
        ("d", path),
        ("transform", transform.as_str()),
        ("stroke", "black"),
        ("stroke-width", "5"),
        #[cfg(feature = "fill")]
        ("fill", FILL_COLOR),
        #[cfg(not(feature = "fill"))]
        ("fill", "none"),
    ];
    writer
        .create_element("path")
        .with_attributes(attrs.into_iter())
        .write_empty()?;
    Ok(())
}

/// Draw the minutes on each side of the grid
fn draw_minutes(
    writer: &mut Writer<BufWriter<File>>,
    c: char,
    font: &FontAnalysis,
    doc: &Sizes,
    scale: f64,
) -> IoResult<()> {
    let glyph = font.glyphs.get(&c).unwrap();
    let path = glyph.path.clone();
    let x_min = glyph.bbox.x_min as f64;
    let x_max = glyph.bbox.x_max as f64;
    let glyph_width = x_max - x_min;
    let glyph_height = (glyph.bbox.y_max - glyph.bbox.y_min) as f64;
    let mid_x = (glyph_width / 2.0 + x_min) * scale;
    let mid_y = glyph_height / 2.0 * scale;

    let x_left = MARGIN / 2.0 - mid_x;
    let x_mid = doc.document_width / 2.0 - mid_x;
    let x_right = doc.document_width - MARGIN / 2.0 - mid_x;
    let y_top = MARGIN / 2.0 + mid_y;
    let y_mid = doc.document_height / 2.0 + mid_y;
    let y_bottom = doc.document_height - MARGIN / 2.0 + mid_y;
    let positions = vec![
        (x_mid, y_top),
        (x_right, y_mid),
        (x_mid, y_bottom),
        (x_left, y_mid),
    ];
    for (x, y) in positions {
        draw_minute(writer, &path, scale, x, y)?;
    }
    Ok(())
}

/// Draw a circle of radius `hole_radius` at position (x, y)
fn draw_hole(writer: &mut Writer<BufWriter<File>>, x: f64, y: f64) -> IoResult<()> {
    let radius_str = (HOLE_DIAMETER / 2.0).to_string();
    let x_str = x.to_string();
    let y_str = y.to_string();
    let attrs = vec![
        ("r", radius_str.as_str()),
        ("cx", x_str.as_str()),
        ("cy", y_str.as_str()),
        ("stroke", "black"),
        ("stroke-width", "5"),
        #[cfg(feature = "fill")]
        ("fill", FILL_COLOR),
        #[cfg(not(feature = "fill"))]
        ("fill", "none"),
    ];
    writer
        .create_element("circle")
        .with_attributes(attrs.into_iter())
        .write_empty()?;
    Ok(())
}

/// Draw the holes on each corner of the grid
fn draw_holes(writer: &mut Writer<BufWriter<File>>, doc: &Sizes) -> IoResult<()> {
    let hole_radius = HOLE_DIAMETER / 2.0;

    let x_left = MARGIN / 2.0 - hole_radius;
    let x_right = doc.document_width - MARGIN / 2.0 - hole_radius;
    let y_top = MARGIN / 2.0 + hole_radius;
    let y_bottom = doc.document_height - hole_radius - MARGIN / 2.0;
    let positions = vec![
        (x_left, y_top),
        (x_right, y_top),
        (x_right, y_bottom),
        (x_left, y_bottom),
    ];
    for (x, y) in positions {
        draw_hole(writer, x, y)?;
    }
    Ok(())
}

/// Write the grid to the SVG file
fn write_grid(
    writer: &mut Writer<BufWriter<File>>,
    font: FontAnalysis,
    doc: &Sizes,
    scale: f64,
) -> IoResult<()> {
    // Compute the horizontal offset to center the LEDs
    let square_width = doc.document_width - 2.0 * MARGIN;
    let right_offset = (square_width - (GRID_WIDTH as f64 - 1.) * LED_SPACING - LED_SIZE) / 2.;
    // Compute the vertical spacing between the LEDs
    let square_height = doc.document_height - 2.0 * MARGIN;
    let vert_spacing = square_height / (GRID_HEIGHT as f64);

    let y_max = font.y_max as f64;

    let base_y = MARGIN + vert_spacing / 2. - LED_SIZE / 2.;
    let base_x = MARGIN + right_offset;
    // Compute the vertical spacing between the LEDs
    let square_height = doc.document_height - 2.0 * MARGIN;
    let vert_spacing = square_height / (GRID_HEIGHT as f64);

    for (y, row) in GRID.iter().enumerate().take(GRID_HEIGHT) {
        let led_y_mid_off = (y as f64 * vert_spacing) + base_y + (LED_SIZE / 2.0);
        let y: f64 = led_y_mid_off + y_max / 2.0 * scale;
        let y_str = y.to_string();

        for (x, c) in row.iter().enumerate().take(GRID_WIDTH) {
            let led_x_mid_off = (x as f64 * LED_SPACING) + base_x + (LED_SIZE / 2.0);

            let glyph = font.glyphs.get(c).unwrap();
            let path = glyph.path.clone();
            let x_min = glyph.bbox.x_min as f64;
            let x_max = glyph.bbox.x_max as f64;
            let glyph_width = x_max - x_min;
            let x = led_x_mid_off - (glyph_width / 2.0 + x_min) * scale;

            let x_str = x.to_string();

            let transform = format!("matrix({} 0 0 {} {} {})", scale, -scale, x_str, y_str);
            let attrs = vec![
                ("d", path.as_str()),
                ("transform", transform.as_str()),
                ("stroke", "black"),
                ("stroke-width", "5"),
                #[cfg(feature = "fill")]
                ("fill", FILL_COLOR),
                #[cfg(not(feature = "fill"))]
                ("fill", "none"),
            ];
            writer
                .create_element("path")
                .with_attributes(attrs.into_iter())
                .write_empty()?;
            #[cfg(feature = "draw_bounding_box")]
            {
                let bb = &glyph.bbox;
                let bbox_path = format!(
                    "M {} {} L {} {} L {} {} L {} {} Z",
                    bb.x_min, bb.y_min, bb.x_max, bb.y_min, bb.x_max, bb.y_max, bb.x_min, bb.y_max
                );
                let bbox_path_attrs = vec![
                    ("d", bbox_path.as_str()),
                    ("transform", transform.as_str()),
                    ("fill", "none"),
                    ("stroke", "black"),
                    ("stroke-width", "3"),
                ];
                writer
                    .create_element("path")
                    .with_attributes(bbox_path_attrs.into_iter())
                    .write_empty()?;
            }
        }
    }
    Ok(())
}

/// Generate the SVG file
pub fn generate(file: &PathBuf, font: FontAnalysis) -> IoResult<()> {
    let mut writer = Writer::new_with_indent(BufWriter::new(File::create(file)?), b' ', 2);
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

    let sizes = Sizes::compute(&font);
    println!(
        "Document size: {}x{}",
        sizes.document_width, sizes.document_height
    );
    let scale = sizes.scale;
    println!("scale:{}", scale);
    let descender = font.descender as f64;
    println!("descender:{}", descender);
    let ascender = font.ascender as f64;
    println!("ascender:{}", ascender);
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
            #[cfg(feature = "draw_leds")]
            draw_leds(writer, &sizes)?;
            #[cfg(feature = "draw_margins")]
            draw_margins(writer, &sizes)?;
            draw_minutes(writer, FLOWER, &font, &sizes, scale)?;
            draw_holes(writer, &sizes)?;
            writer
                .create_element("g")
                .with_attributes(vec![("id", "grid")].into_iter())
                .write_inner_content(|w| write_grid(w, font, &sizes, scale))?;
            Ok(())
        })?;
    Ok(())
}

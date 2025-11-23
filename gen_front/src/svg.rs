//! Generate the SVG file from a Cover structure

use crate::cover::{Circle, Cover, Path, Shape};
use quick_xml::events::{BytesDecl, Event};
use quick_xml::Writer;
use std::fs::File;
use std::io::BufWriter;
use std::io::Result as IoResult;
use std::path::PathBuf;

const FILL_COLOR: &str = "black";
//const FILL_COLOR: &str = "darkorange";

///// Draw the LEDs in the grid
//#[cfg(feature = "draw_leds")]
//fn draw_leds(writer: &mut Writer<BufWriter<File>>, doc: &Sizes) -> IoResult<()> {
//    // Compute the horizontal offset to center the LEDs
//    let square_width = doc.document_width - 2.0 * MARGIN;
//    let right_offset = (square_width - (GRID_WIDTH as f64 - 1.) * LED_SPACING - LED_SIZE) / 2.;
//    // Compute the vertical spacing between the LEDs
//    let square_height = doc.document_height - 2.0 * MARGIN;
//    let vert_spacing = square_height / (GRID_HEIGHT as f64);
//
//    let base_y = MARGIN + vert_spacing / 2. - LED_SIZE / 2.;
//    let base_x = MARGIN + right_offset;
//    let led_size = LED_SIZE.to_string();
//    for y in 0..GRID_HEIGHT {
//        for x in 0..GRID_WIDTH {
//            let x_str = (x as f64 * LED_SPACING + base_x).to_string();
//            let y_str = (y as f64 * vert_spacing + base_y).to_string();
//            let attrs = vec![
//                ("x", x_str.as_str()),
//                ("y", y_str.as_str()),
//                ("width", led_size.as_str()),
//                ("height", led_size.as_str()),
//                ("fill", "red"),
//            ];
//            writer
//                .create_element("rect")
//                .with_attributes(attrs.into_iter())
//                .write_empty()?;
//        }
//    }
//    Ok(())
//}

/// Draw a circle
fn draw_circle(
    writer: &mut Writer<BufWriter<File>>,
    circle: Circle,
    _fill_color: &str,
) -> IoResult<()> {
    let radius_str = circle.r.to_string();
    let x_str = circle.cx.to_string();
    let y_str = circle.cy.to_string();
    let attrs = vec![
        ("r", radius_str.as_str()),
        ("cx", x_str.as_str()),
        ("cy", y_str.as_str()),
        ("stroke", "black"),
        ("stroke-width", "5"),
        #[cfg(feature = "fill")]
        ("fill", _fill_color),
        #[cfg(not(feature = "fill"))]
        ("fill", "none"),
    ];
    writer
        .create_element("circle")
        .with_attributes(attrs.into_iter())
        .write_empty()?;
    Ok(())
}

/// Draw a path
fn draw_path(
    writer: &mut Writer<BufWriter<File>>,
    path: Path,
    scale: f64,
    _fill_color: &str,
) -> IoResult<()> {
    // Position on top side, in the middlej
    // Position on the left side
    let transform = format!("matrix({} 0 0 {} {} {})", scale, -scale, path.x, path.y);
    let attrs = vec![
        ("d", path.d.as_str()),
        ("transform", transform.as_str()),
        ("stroke", "black"),
        ("stroke-width", "5"),
        #[cfg(feature = "fill")]
        ("fill", _fill_color),
        #[cfg(not(feature = "fill"))]
        ("fill", "none"),
    ];
    writer
        .create_element("path")
        .with_attributes(attrs.into_iter())
        .write_empty()?;
    Ok(())
}

/// Draw a shape
fn draw_shape(
    writer: &mut Writer<BufWriter<File>>,
    shape: Shape,
    scale: f64,
    fill_color: &str,
) -> IoResult<()> {
    match shape {
        Shape::Circle(c) => {
            draw_circle(writer, c, fill_color)?;
        }
        Shape::Path(p) => {
            draw_path(writer, p, scale, fill_color)?;
        }
    }
    Ok(())
}

/// Generate the SVG file
pub fn generate(file: &PathBuf, cover: Cover) -> IoResult<()> {
    let mut writer = Writer::new_with_indent(BufWriter::new(File::create(file)?), b' ', 2);
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

    let scale = cover.scale;
    println!("scale:{}", scale);
    let width_mm = format!("{}mm", cover.width);
    let height_mm = format!("{}mm", cover.height);
    let view_box = format!("0 0 {} {}", cover.width, cover.height);
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
            for shape in cover.shapes {
                draw_shape(writer, shape, scale, FILL_COLOR)?;
            }
            //#[cfg(feature = "draw_leds")]
            //draw_leds(writer, &sizes)?;
            //#[cfg(feature = "draw_margins")]
            //draw_margins(writer, &sizes)?;
            //draw_minutes(writer, FLOWER, &font, &sizes, scale)?;
            //draw_holes(writer, &sizes)?;
            //writer
            //    .create_element("g")
            //    .with_attributes(vec![("id", "grid")].into_iter())
            //    .write_inner_content(|w| write_grid(w, font, &sizes, scale))?;
            Ok(())
        })?;
    Ok(())
}

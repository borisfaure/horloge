/// Generates a DXF file for the given cover design.
use crate::cover::{Circle, Cover, Path, Shape};
use std::fs::File;
use std::io::Result as IoResult;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Debug, Clone, Copy)]
struct Matrix {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    e: f64,
    f: f64,
}

fn write_dxf_header(writer: &mut BufWriter<File>, width: f64, height: f64) -> IoResult<()> {
    writeln!(writer, "0")?;
    writeln!(writer, "SECTION")?;
    writeln!(writer, "2")?;
    writeln!(writer, "HEADER")?;

    // $INSUNITS to millimeters
    writeln!(writer, "9")?;
    writeln!(writer, "$INSUNITS")?;
    writeln!(writer, "70")?;
    writeln!(writer, "4")?;

    // Set drawing extents (bounding box)
    writeln!(writer, "9")?;
    writeln!(writer, "$EXTMIN")?;
    writeln!(writer, "10")?;
    writeln!(writer, "0.0")?;
    writeln!(writer, "20")?;
    writeln!(writer, "0.0")?;
    writeln!(writer, "30")?;
    writeln!(writer, "0.0")?;

    writeln!(writer, "9")?;
    writeln!(writer, "$EXTMAX")?;
    writeln!(writer, "10")?;
    writeln!(writer, "{}", width)?;
    writeln!(writer, "20")?;
    writeln!(writer, "{}", height)?;
    writeln!(writer, "30")?;
    writeln!(writer, "0.0")?;

    // Set limits
    writeln!(writer, "9")?;
    writeln!(writer, "$LIMMIN")?;
    writeln!(writer, "10")?;
    writeln!(writer, "0.0")?;
    writeln!(writer, "20")?;
    writeln!(writer, "0.0")?;

    writeln!(writer, "9")?;
    writeln!(writer, "$LIMMAX")?;
    writeln!(writer, "10")?;
    writeln!(writer, "{}", width)?;
    writeln!(writer, "20")?;
    writeln!(writer, "{}", height)?;

    writeln!(writer, "0")?;
    writeln!(writer, "ENDSEC")?;

    Ok(())
}

fn write_dxf_tables(writer: &mut BufWriter<File>) -> IoResult<()> {
    writeln!(writer, "0")?;
    writeln!(writer, "SECTION")?;
    writeln!(writer, "2")?;
    writeln!(writer, "TABLES")?;

    // Layer table
    writeln!(writer, "0")?;
    writeln!(writer, "TABLE")?;
    writeln!(writer, "2")?;
    writeln!(writer, "LAYER")?;
    writeln!(writer, "70")?;
    writeln!(writer, "1")?;

    // Default layer
    writeln!(writer, "0")?;
    writeln!(writer, "LAYER")?;
    writeln!(writer, "2")?;
    writeln!(writer, "0")?;
    writeln!(writer, "70")?;
    writeln!(writer, "0")?;
    writeln!(writer, "62")?;
    writeln!(writer, "7")?;
    writeln!(writer, "6")?;
    writeln!(writer, "CONTINUOUS")?;

    writeln!(writer, "0")?;
    writeln!(writer, "ENDTAB")?;
    writeln!(writer, "0")?;
    writeln!(writer, "ENDSEC")?;

    Ok(())
}

fn write_dxf_entities_start(writer: &mut BufWriter<File>) -> IoResult<()> {
    writeln!(writer, "0")?;
    writeln!(writer, "SECTION")?;
    writeln!(writer, "2")?;
    writeln!(writer, "ENTITIES")?;

    Ok(())
}

fn write_dxf_entities_end(writer: &mut BufWriter<File>) -> IoResult<()> {
    writeln!(writer, "0")?;
    writeln!(writer, "ENDSEC")?;

    Ok(())
}

fn write_dxf_footer(writer: &mut BufWriter<File>) -> IoResult<()> {
    writeln!(writer, "0")?;
    writeln!(writer, "EOF")?;

    Ok(())
}

/// Write a circle in DXF format
fn write_circle(writer: &mut BufWriter<File>, c: Circle) -> IoResult<()> {
    writeln!(writer, "0")?;
    writeln!(writer, "CIRCLE")?;
    writeln!(writer, "8")?; // Layer
    writeln!(writer, "0")?;
    writeln!(writer, "10")?; // Center X
    writeln!(writer, "{}", c.cx)?;
    writeln!(writer, "20")?; // Center Y
    writeln!(writer, "{}", c.cy)?;
    writeln!(writer, "30")?; // Center Z
    writeln!(writer, "0.0")?;
    writeln!(writer, "40")?; // Radius
    writeln!(writer, "{}", c.r)?;

    Ok(())
}

/// Write a rectangle as 4 lines in DXF format
fn write_rectangle(
    writer: &mut BufWriter<File>,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> IoResult<()> {
    // Draw rectangle as 4 lines (bottom, right, top, left)
    write_line(writer, x, y, x + width, y)?; // Bottom
    write_line(writer, x + width, y, x + width, y + height)?; // Right
    write_line(writer, x + width, y + height, x, y + height)?; // Top
    write_line(writer, x, y + height, x, y)?; // Left

    Ok(())
}

/// Write a line in DXF format
fn write_line(writer: &mut BufWriter<File>, x1: f64, y1: f64, x2: f64, y2: f64) -> IoResult<()> {
    writeln!(writer, "0")?;
    writeln!(writer, "LINE")?;
    writeln!(writer, "8")?; // Layer
    writeln!(writer, "0")?;
    writeln!(writer, "10")?; // Start X
    writeln!(writer, "{}", x1)?;
    writeln!(writer, "20")?; // Start Y
    writeln!(writer, "{}", y1)?;
    writeln!(writer, "30")?; // Start Z
    writeln!(writer, "0.0")?;
    writeln!(writer, "11")?; // End X
    writeln!(writer, "{}", x2)?;
    writeln!(writer, "21")?; // End Y
    writeln!(writer, "{}", y2)?;
    writeln!(writer, "31")?; // End Z
    writeln!(writer, "0.0")?;

    Ok(())
}

fn parse_svg_path(path_data: &str, transform: &Matrix) -> Vec<Vec<Point>> {
    let mut points = Vec::new();
    let tokens: Vec<&str> = path_data.split_whitespace().collect();
    let mut i = 0;
    let mut splines = Vec::new();

    while i < tokens.len() {
        let cmd = tokens[i];
        i += 1;

        match cmd {
            "M" => {
                // Move to
                if i + 1 < tokens.len() {
                    let x: f64 = tokens[i].parse().unwrap_or(0.0);
                    let y: f64 = tokens[i + 1].parse().unwrap_or(0.0);
                    points.push(transform_point(x, y, transform));
                    i += 2;
                }
            }
            "L" => {
                // Line to
                if i + 1 < tokens.len() {
                    let x: f64 = tokens[i].parse().unwrap_or(0.0);
                    let y: f64 = tokens[i + 1].parse().unwrap_or(0.0);
                    points.push(transform_point(x, y, transform));
                    i += 2;
                }
            }
            "Q" => {
                // Quadratic Bezier curve
                if i + 3 < tokens.len() {
                    let cx: f64 = tokens[i].parse().unwrap_or(0.0);
                    let cy: f64 = tokens[i + 1].parse().unwrap_or(0.0);
                    let x: f64 = tokens[i + 2].parse().unwrap_or(0.0);
                    let y: f64 = tokens[i + 3].parse().unwrap_or(0.0);

                    let control = transform_point(cx, cy, transform);
                    let end = transform_point(x, y, transform);

                    // Add control point and end point for spline
                    points.push(control);
                    points.push(end);
                    i += 4;
                }
            }
            "Z" | "z" => {
                // Close path - connect to first point
                if !points.is_empty() {
                    points.push(points[0]);
                }
                splines.push(points.clone());
                points.clear();
            }
            _ => {
                // Try to parse as number (implicit line continuation)
                if let Ok(x) = cmd.parse::<f64>() {
                    if i < tokens.len() {
                        let y: f64 = tokens[i].parse().unwrap_or(0.0);
                        points.push(transform_point(x, y, transform));
                        i += 1;
                    }
                }
            }
        }
    }

    if !points.is_empty() {
        splines.push(points);
    }

    splines
}

fn transform_point(x: f64, y: f64, m: &Matrix) -> Point {
    Point {
        x: m.a * x + m.c * y + m.e,
        y: m.b * x + m.d * y + m.f,
    }
}

fn quadratic_bezier(p0: Point, p1: Point, p2: Point, t: f64) -> Point {
    let t2 = 1.0 - t;
    Point {
        x: t2 * t2 * p0.x + 2.0 * t2 * t * p1.x + t * t * p2.x,
        y: t2 * t2 * p0.y + 2.0 * t2 * t * p1.y + t * t * p2.y,
    }
}

fn write_spline(writer: &mut BufWriter<File>, points: &[Point]) -> IoResult<()> {
    if points.is_empty() {
        return Ok(());
    }

    let n = points.len();
    let degree = 3; // Cubic spline
    let num_knots = n + degree + 1;

    writeln!(writer, "0")?;
    writeln!(writer, "SPLINE")?;
    writeln!(writer, "8")?; // Layer
    writeln!(writer, "0")?;
    writeln!(writer, "70")?; // Spline flag (8 = closed spline)
    writeln!(writer, "8")?;
    writeln!(writer, "71")?; // Degree of spline
    writeln!(writer, "{}", degree)?;
    writeln!(writer, "72")?; // Number of knots
    writeln!(writer, "{}", num_knots)?;
    writeln!(writer, "73")?; // Number of control points
    writeln!(writer, "{}", n)?;
    writeln!(writer, "74")?; // Number of fit points
    writeln!(writer, "0")?;

    // Write knot values (uniform knot vector for closed spline)
    for i in 0..num_knots {
        writeln!(writer, "40")?;
        writeln!(writer, "{}", i)?;
    }

    // Write control points
    for point in points {
        writeln!(writer, "10")?;
        writeln!(writer, "{}", point.x)?;
        writeln!(writer, "20")?;
        writeln!(writer, "{}", point.y)?;
        writeln!(writer, "30")?;
        writeln!(writer, "0.0")?;
    }

    Ok(())
}

/// Write a path
fn write_path(writer: &mut BufWriter<File>, height: f64, path: Path, scale: f64) -> IoResult<()> {
    let transform = Matrix {
        a: scale,
        b: 0.0,
        c: 0.0,
        d: scale,
        e: path.x,
        f: height - path.y,
    };
    let points_vec = parse_svg_path(path.d.as_str(), &transform);
    for segment in points_vec {
        write_spline(writer, &segment)?;
    }
    Ok(())
}

/// Write a shape
fn write_shape(
    writer: &mut BufWriter<File>,
    height: f64,
    shape: Shape,
    scale: f64,
) -> IoResult<()> {
    match shape {
        Shape::Circle(c) => {
            write_circle(writer, c)?;
        }
        Shape::Path(p) => {
            write_path(writer, height, p, scale)?;
        }
    }
    Ok(())
}

pub fn generate(file: &PathBuf, cover: Cover) -> IoResult<()> {
    // Placeholder implementation for DXF generation
    println!("Generating DXF file at: {:?}", file);
    let mut writer = BufWriter::new(File::create(file)?);

    let scale = cover.scale;
    println!("scale:{}", scale);

    // Define geometry parameters
    let width = cover.width;
    let height = cover.height;

    write_dxf_header(&mut writer, width, height)?;
    write_dxf_tables(&mut writer)?;
    write_dxf_entities_start(&mut writer)?;

    // Draw outer rectangle
    write_rectangle(&mut writer, 0.0, 0.0, width, height)?;

    for shape in cover.shapes {
        write_shape(&mut writer, height, shape, scale)?;
    }

    write_dxf_entities_end(&mut writer)?;
    write_dxf_footer(&mut writer)?;

    writer.flush()?;
    Ok(())
}

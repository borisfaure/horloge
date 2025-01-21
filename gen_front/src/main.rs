use clap::{Arg, Command};
use quick_xml::events::{BytesDecl, BytesText, Event};
use quick_xml::Writer;
use std::fs::File;
use std::io::BufWriter;
use std::io::Result as IoResult;
use std::path::PathBuf;

/// Clap command definition
fn command() -> Command {
    Command::new("myapp")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Boris Faure <boris@fau.re>")
        .about("Generate an SVG file with a custom grid")
        .arg(
            Arg::new("FILE")
                .value_name("FILE")
                .num_args(1)
                .required(true)
                .value_parser(clap::builder::PathBufValueParser::new())
                .help("File to write the SVG to"),
        )
}

const FONT_FAMILY: &str = "Siruca";
// Letter N: 31.45mm width, 51mm height
const LETTER_RATIO: f64 = 51.0 / 31.45;
const LED_SIZE: f64 = 5f64;
const LED_SPACING: f64 = 12f64 + LED_SIZE;
const FONT_SIZE: usize = 24;

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

/// Write the grid to the SVG file
fn write_grid(writer: &mut Writer<BufWriter<File>>) -> IoResult<()> {
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let x_str = (x as f64 * LED_SPACING).to_string();
            let y_str = (y * 30).to_string();
            let font_size = FONT_SIZE.to_string();
            let attrs = vec![
                ("x", x_str.as_str()),
                ("y", y_str.as_str()),
                ("font-size", font_size.as_str()),
                ("font-family", FONT_FAMILY),
                ("fill", "black"),
            ];
            writer
                .create_element("text")
                .with_attributes(attrs.into_iter())
                .write_text_content(BytesText::new(GRID[y][x].to_string().as_str()))?;
        }
    }
    Ok(())
}

#[derive(Debug)]
struct Sizes {
    document_width: f64,
    document_height: f64,
}

impl Sizes {
    fn compute() -> Sizes {
        // We need to compute the best font size so that the grid is square
        // We know the spacing between the LEDs, the size of the LEDs, and the
        // ratio of the letters
        //
        //
        // Let l be the width of the letter, h the height of the letter.
        // k = l / h, or h = l / k
        // The visual space (v) between two letters is the spacing between two
        // LEDs, plus l.
        // We want the grid to be visually a square.
        // GRID_WIDTH * LED_SPACING + l = GRID_HEIGHT * h + (GRID_HEIGHT - 1) * h

        let d = LED_SPACING;
        let k = LETTER_RATIO;
        let h = GRID_HEIGHT as f64;
        let w = GRID_WIDTH as f64;

        let l = d * (w + 1f64 - h) / (h * (k - 1f64));
        let v = d - l;
        println!("Letter width: {}", l);
        println!("Letter height: {}", l / k);
        println!("Letter spacing: {}", v);

        let document_width = w * l + (w - 1f64) * v + 2f64 * MARGIN;
        let document_height = h * l + (h - 1f64) * v + 2f64 * MARGIN;

        Sizes {
            document_width,
            document_height,
        }
    }
}

/// Generate the SVG file
fn gen_xml(file: &PathBuf) -> IoResult<()> {
    let mut writer = Writer::new_with_indent(BufWriter::new(File::create(file)?), b' ', 2);
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

    let sizes = Sizes::compute();
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
                .write_inner_content(|writer| write_grid(writer))?;
            Ok(())
        })?;
    Ok(())
}

fn main() {
    let matches = command() // requires `cargo` feature
        .get_matches();

    let file = matches.get_one::<PathBuf>("FILE").unwrap();
    gen_xml(file).unwrap();
}

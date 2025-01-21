use clap::{Arg, Command};
use std::path::PathBuf;

mod font;
mod svg;

/// Clap command definition
fn command() -> Command {
    Command::new("myapp")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Boris Faure <boris@fau.re>")
        .about("Generate an SVG file with a custom grid")
        .arg(
            Arg::new("TTF")
                .value_name("TTF")
                .num_args(1)
                .required(true)
                .value_parser(clap::builder::PathBufValueParser::new())
                .help("TTF file to use for the font"),
        )
        .arg(
            Arg::new("SVG")
                .value_name("SVG")
                .num_args(1)
                .required(true)
                .value_parser(clap::builder::PathBufValueParser::new())
                .help("File to write the SVG to"),
        )
}

fn main() {
    let matches = command() // requires `cargo` feature
        .get_matches();

    let font = matches.get_one::<PathBuf>("TTF").unwrap();
    let font_data = std::fs::read(font).unwrap();
    let fa = font::analyze_font(font_data).unwrap();
    println!("Font: {:?}", fa);

    let svg = matches.get_one::<PathBuf>("SVG").unwrap();
    svg::generate(svg).unwrap();
}

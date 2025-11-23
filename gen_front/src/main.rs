use clap::{Arg, Command};
use std::path::PathBuf;

mod cover;
mod dxf;
mod font;
mod svg;

/// Clap command definition
fn command() -> Command {
    Command::new("gen_front")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Boris Faure <boris@fau.re>")
        .about("Generate an SVG or a DXF file with a custom grid")
        .arg(
            Arg::new("TTF")
                .value_name("TTF")
                .num_args(1)
                .required(true)
                .value_parser(clap::builder::PathBufValueParser::new())
                .help("TTF file to use for the font"),
        )
        .arg(
            Arg::new("FILE")
                .value_name("FILE")
                .num_args(1)
                .required(true)
                .value_parser(clap::builder::PathBufValueParser::new())
                .help("File to write the frame to: must end with .svg or .dxf"),
        )
}

fn main() {
    let matches = command() // requires `cargo` feature
        .get_matches();

    let font = matches.get_one::<PathBuf>("TTF").unwrap();
    let font_data = std::fs::read(font).unwrap();
    let fa = font::analyze_font(font_data).unwrap();
    println!("y_max:{}", fa.y_max);
    println!("glyph_width_avg:{}", fa.glyph_width_avg);

    let file = matches.get_one::<PathBuf>("FILE").unwrap();
    if file.extension().is_none() {
        panic!("File must have an extension");
    }
    let ext = file.extension().unwrap().to_str().unwrap();
    match ext {
        "svg" => {}
        "dxf" => {}
        _ => panic!("File must end with .svg or .dxf"),
    }
    let cover = cover::Cover::new(fa);
    match ext {
        "svg" => svg::generate(file, cover).unwrap(),
        "dxf" => dxf::generate(file, cover).unwrap(),
        _ => unreachable!(),
    }
}

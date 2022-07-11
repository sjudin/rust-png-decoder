use pyo3::prelude::*;

mod decoder;
mod parser;

use crate::parser::{parse_png, Color};

fn parse_and_decode_png(path: &String) -> Vec<Vec<Color>> {
    let png_image = match parse_png(path) {
        Ok(png) => png,
        Err(error) => panic!("An error occured while parsing png file: \"{}\"", error),
    };

    match decoder::decode_png(&png_image) {
        Some(image) => image,
        None => panic!("This format is not supported yet!"),
    }
}

/// Read and decode a png file and return a two-dimensional vector of RGB values
#[pyfunction]
fn read_png(path: String) -> PyResult<Vec<Vec<(u8, u8, u8)>>> {
    let mut res: Vec<Vec<(u8, u8, u8)>> = Vec::new();
    let img = parse_and_decode_png(&path);

    for row in img {
        let mut tmp: Vec<(u8, u8, u8)> = Vec::new();
        for pixel in row {
            tmp.push((pixel.red, pixel.green, pixel.blue));
        }
        res.push(tmp);
    }

    Ok(res)
}

/// Read and decode a png file and return a two-dimensional vector of RGB values
pub fn read_and_print_png(path: &String) {
    let img = parse_and_decode_png(path);
    decoder::print_png(&img);
}

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn rust_png_reader(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(read_png, m)?)?;
    Ok(())
}

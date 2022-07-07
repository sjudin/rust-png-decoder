use pyo3::prelude::*;

mod decoder;
mod parser;

#[pyfunction]
fn read_png(path: String) -> PyResult<Vec<Vec<(u8, u8, u8)>>> {
    let mut res: Vec<Vec<(u8, u8, u8)>> = Vec::new();

    let png_file: parser::PngImage = parser::parse_png(&path).unwrap();
    let img = decoder::png_indexed_color_to_pixels(&png_file);

    for row in img {
        let mut tmp: Vec<(u8, u8, u8)> = Vec::new();
        for pixel in row {
            tmp.push((pixel.red, pixel.green, pixel.blue));
        }
        res.push(tmp);
    }

    Ok(res)
}

pub fn read_and_print_png(path: &String) {
    let png_file: parser::PngImage = parser::parse_png(path).unwrap();
    let img = decoder::png_indexed_color_to_pixels(&png_file);
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

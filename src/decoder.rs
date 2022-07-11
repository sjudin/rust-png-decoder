use crate::parser::{Color, ColorType, PngImage};
use colored::Colorize;

/// Decodes a png image and return the result using one of the decoder functions.
/// If the png format is not supported then None is returned
pub fn decode_png(png_image: &PngImage) -> Option<Vec<Vec<Color>>> {
    match png_image.color_type {
        ColorType::Truecolor | ColorType::TrueColorWithAlpha => {
            Some(png_truecolor_to_pixels(png_image))
        }
        ColorType::IndexedColor => Some(png_indexed_color_to_pixels(png_image)),
        ColorType::Grayscale => Some(png_grayscale_to_pixels(png_image)),
        _ => None,
    }
}

/// Uses the Colorize crate to print a png image to the terminal as RGB.
/// Requires a terminal with truecolor support
pub fn print_png(pixel_data: &Vec<Vec<Color>>) {
    for row in pixel_data {
        for pixel in row {
            print!("{}", " ".on_truecolor(pixel.red, pixel.green, pixel.blue));
        }
        println!();
    }
}

/// Decode pixels of a parsed png image assumed to follow a indexed color
/// format, return Vec<Vec<Color>>, the vectors represent the rows and columns
/// respectively
fn png_indexed_color_to_pixels(png_file: &PngImage) -> Vec<Vec<Color>> {
    let mut res: Vec<Vec<Color>> = Vec::new();
    let bits_per_scanline = (png_file.width * png_file.bit_depth as u32) as usize;

    let bytes_per_scanline = (bits_per_scanline as f32 / 8.0).ceil() as usize;
    let palette = png_file.palette.as_ref().unwrap();
    let mask = ((1_u16 << png_file.bit_depth) - 1) as u8;

    // Each scanline
    for scanline_idx in (0..png_file.data.len()).step_by(bytes_per_scanline) {
        let mut scanline: Vec<Color> = Vec::new();
        let mut bits_parsed = 0;

        // Iterate over each byte in the scanline
        for byte_idx in scanline_idx..scanline_idx + bytes_per_scanline {
            for bit_idx in (0..8).step_by(png_file.bit_depth as usize).rev() {
                let palette_idx: usize = (png_file.data[byte_idx] >> bit_idx & mask).into();
                scanline.push(Color {
                    ..palette[palette_idx]
                });
                bits_parsed += png_file.bit_depth as usize;
                if bits_parsed == bits_per_scanline {
                    break;
                }
            }
        }
        res.push(scanline);
    }
    res
}

/// Decode pixels of a parsed png image assumed to follow a grayscale color
/// format, return Vec<Vec<Color>>, the vectors represent the rows and columns
/// respectively
fn png_grayscale_to_pixels(png_file: &PngImage) -> Vec<Vec<Color>> {
    let mut res: Vec<Vec<Color>> = Vec::new();

    let bits_per_scanline = (png_file.width * png_file.bit_depth as u32) as usize;
    let bytes_per_scanline = (bits_per_scanline as f32 / 8.0).ceil() as usize;

    let mask = ((1_u16 << png_file.bit_depth) - 1) as u8;

    let scale_factor = match png_file.bit_depth {
        1 => 255,
        2 => 85,
        4 => 17,
        8 => 1,
        _ => 0,
    };

    // Each scanline
    for scanline_idx in (0..png_file.data.len()).step_by(bytes_per_scanline) {
        let mut scanline: Vec<Color> = Vec::new();
        let mut bits_parsed = 0;

        // Iterate over each byte in the scanline
        for byte_idx in scanline_idx..scanline_idx + bytes_per_scanline {
            for bit_idx in (0..8).step_by(png_file.bit_depth as usize).rev() {
                let val: u8 = png_file.data[byte_idx] >> bit_idx & mask;
                scanline.push(Color {
                    red: val * scale_factor,
                    green: val * scale_factor,
                    blue: val * scale_factor,
                });
                bits_parsed += png_file.bit_depth as usize;
                if bits_parsed == bits_per_scanline {
                    break;
                }
            }
        }
        res.push(scanline);
    }
    res
}

/// Decode pixels of a parsed png image assumed to follow a truecolor png
/// format, return Vec<Vec<Color>>, the vectors represent the rows and columns
/// respectively. Note that the alpha channel is ignored for Truecolor images
/// with alpha
fn png_truecolor_to_pixels(png_file: &PngImage) -> Vec<Vec<Color>> {
    let mut res: Vec<Vec<Color>> = Vec::new();
    let bytes_per_channel = png_file.bit_depth as usize / 8;
    let bytes_per_pixel: usize = match png_file.color_type {
        ColorType::Truecolor => 3,
        ColorType::TrueColorWithAlpha => 4,
        _ => panic!(),
    };
    let bytes_per_scanline = bytes_per_pixel * png_file.width as usize * bytes_per_channel;

    for scanline_idx in 0..png_file.height as usize {
        let mut scanline: Vec<Color> = Vec::new();

        for pixel_idx in (0..bytes_per_scanline).step_by(bytes_per_channel * bytes_per_pixel) {
            let pixel_start = scanline_idx * bytes_per_scanline + pixel_idx;
            let red_idx = pixel_start;
            let green_idx = pixel_start + bytes_per_channel;
            let blue_idx = pixel_start + bytes_per_channel * 2;

            // Decode the RGB value
            let red: u8 = png_file.data[red_idx];
            let green: u8 = png_file.data[green_idx];
            let blue: u8 = png_file.data[blue_idx];

            scanline.push(Color { red, green, blue });
        }
        res.push(scanline);
    }
    res
}

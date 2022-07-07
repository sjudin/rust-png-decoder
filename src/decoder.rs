#![allow(dead_code)]

use crate::parser::{Color, PngImage};
use colored::Colorize;

/// Decode pixels of a parsed png image assumed to follow a indexed color
/// format, return Vec<Vec<Color>>, the vectors represent the rows and columns
/// respectively
pub fn png_indexed_color_to_pixels(png_file: &PngImage) -> Vec<Vec<Color>> {
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

/// Prints a
pub fn print_png(pixel_data: &Vec<Vec<Color>>) {
    for row in pixel_data {
        for pixel in row {
            print!("{}", " ".on_truecolor(pixel.red, pixel.green, pixel.blue));
        }
        println!();
    }
}

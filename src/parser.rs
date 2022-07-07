#![allow(dead_code)]

use std::io::Read;

pub type Result<T> = std::result::Result<T, PngError>;

#[derive(Debug)]
pub enum PngError {
    CouldNotReadFile,
    ChecksumFailure,
    NotAPng,
    WrongFormat(String),
    FilterNotSupported(u8),
    DecompressionFailed,
    NotSupported(String),
}

impl std::error::Error for PngError {}

impl std::fmt::Display for PngError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PngError::CouldNotReadFile => write!(f, "Could not read the file"),
            PngError::ChecksumFailure => write!(f, "Checksum incorrect"),
            PngError::NotAPng => write!(f, "File is not png format"),
            PngError::WrongFormat(t) => write!(f, "Incorrect png format: {}", t),
            PngError::FilterNotSupported(t) => write!(f, "Filter type {} not supported", t),
            PngError::DecompressionFailed => write!(f, "Decompression failed!"),
            PngError::NotSupported(t) => write!(f, "Not supported: {}", t),
        }
    }
}

#[derive(Debug)]
enum ColorType {
    Grayscale,
    Truecolor,
    IndexedColor,
    GrayScaleWithAlpha,
    TrueColorWithAlpha,
}

#[derive(Debug)]
enum CompressionMethod {
    DeflateInflate,
}

#[derive(Debug)]
enum FilterMethod {
    FiveTypeAdaptive,
}

#[derive(Debug)]
enum InterlaceMethod {
    NoInterlace,
    Adam7Interlace,
}

#[derive(Debug)]
enum ChunkType {
    Ihrd,
    Plte,
    Idat,
    Iend,
    Ancillary(String),
}

/// Calculate crc32 checksum for the bytes in seq, pretty much stolen from
/// here: https://lxp32.github.io/docs/a-simple-example-crc32-calculation/
fn crc32(seq: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;

    for ch in seq.iter() {
        let mut c = *ch as u32;
        for _ in 0..8 {
            let b = (c ^ crc) & 1;
            crc >>= 1;
            if b > 0 {
                crc ^= 0xEDB88320;
            }
            c >>= 1;
        }
    }
    !crc
}

#[derive(Debug)]
/// struct representing a raw chunk of a png file
struct Chunk {
    length: u32,
    chunk_type: ChunkType,
    chunk_data: Option<Vec<u8>>,
}

impl Chunk {
    /// Construct a Chunk from a buffer and a starting index
    fn from_buffer_index(idx: usize, buf: &Vec<u8>) -> Result<Chunk> {
        if idx + 12 > buf.len() {
            return Err(PngError::WrongFormat(
                "Buffer containing the image is short".to_string(),
            ));
        }

        let length = u32::from_be_bytes(buf[idx..idx + 4].try_into().unwrap());
        let chunk_type = match std::str::from_utf8(&buf[idx + 4..idx + 8]).unwrap() {
            "IHDR" => Ok(ChunkType::Ihrd),
            "PLTE" => Ok(ChunkType::Plte),
            "IDAT" => Ok(ChunkType::Idat),
            "IEND" => Ok(ChunkType::Iend),
            other => Ok(ChunkType::Ancillary(other.to_string())),
        }?;

        let chunk_data = {
            if length > 0 {
                Some(buf[idx + 8..idx + 8 + (length as usize)].to_vec())
            } else {
                None
            }
        };

        let crc = u32::from_be_bytes(
            buf[idx + 8 + (length as usize)..idx + 12 + (length as usize)]
                .try_into()
                .unwrap(),
        );

        // Calculate checksum and verify that it is correct
        if crc32(&buf[idx + 4..idx + 8 + length as usize]) != crc {
            return Err(PngError::ChecksumFailure);
        }

        Ok(Chunk {
            length,
            chunk_type,
            chunk_data,
        })
    }
}

#[derive(Debug)]
/// Color representation as RGB
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Debug)]
/// Representation of a png image file
pub struct PngImage {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    color_type: ColorType,
    compression_method: CompressionMethod,
    filter_method: FilterMethod,
    interlace_method: InterlaceMethod,
    pub palette: Option<Vec<Color>>,
    pub data: Vec<u8>,
}

/// Check the png magic header and return () if the buffer contains a .png file,
/// otherwise return an error
fn check_if_png(buffer_with_image: &[u8]) -> Result<()> {
    let png_header: Vec<u8> = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    let buffer_header = &buffer_with_image[..8];

    let matching = png_header
        .iter()
        .zip(buffer_header)
        .filter(|&(a, b)| a == b)
        .count();

    match matching {
        8 => Ok(()),
        _ => Err(PngError::NotAPng),
    }
}

/// Read a file, check that it is a .png file and return a buffer containing the
/// contents of the file
fn read_file(path: &String) -> Result<Vec<u8>> {
    let mut f = std::fs::File::open(path).map_err(|_| PngError::CouldNotReadFile)?;

    let mut buffer: Vec<u8> = Vec::new();

    f.read_to_end(&mut buffer)
        .map_err(|_| PngError::CouldNotReadFile)?;

    check_if_png(&buffer)?;

    Ok(buffer)
}

/// Parse all chunks contained in a png file and return a Vec<Chunk> containing
/// them
fn parse_chunks(img_buf: &Vec<u8>) -> Result<Vec<Chunk>> {
    let mut res: Vec<Chunk> = Vec::new();

    // png data begins at index 8
    let mut idx = 8;

    loop {
        let c = Chunk::from_buffer_index(idx, img_buf)?;
        idx += (c.length as usize) + 12; // 4 (chunk_length) + 4 (chunk_type)
                                         // + 4 (crc)
        res.push(c);
        // Check the last chunk type we pushed, if it is IEND we stop here
        // and return the read chunks
        if matches!(res.last().unwrap().chunk_type, ChunkType::Iend) {
            break Ok(res);
        }
    }
}

/// Find a PLTE block among the chunks and parse the palette colors, if
/// no PLTE block is present return None
fn parse_palette(chunks: &Vec<Chunk>) -> Option<Vec<Color>> {
    let mut res: Vec<Color> = Vec::new();
    for chunk in chunks {
        // Palette chunk found, parse it
        if matches!(chunk.chunk_type, ChunkType::Plte) {
            let color_data = chunk.chunk_data.as_ref().unwrap();
            for idx in (0..color_data.len()).step_by(3) {
                res.push(Color {
                    red: color_data[idx],
                    green: color_data[idx + 1],
                    blue: color_data[idx + 2],
                })
            }
            return Some(res);
        }
    }
    // We did not find a Palette chunk, return None
    None
}

/// Go over all IDAT blocks among the chunks and concatenate all the blocks
/// into a single Vec<u8>
fn collect_idat_data(chunks: Vec<Chunk>) -> Vec<u8> {
    let mut res: Vec<u8> = Vec::new();

    for chunk in chunks {
        if matches!(chunk.chunk_type, ChunkType::Idat) {
            res.append(&mut chunk.chunk_data.unwrap());
        }
    }
    res
}

/// Decompress data and return it
fn decompress(data: &Vec<u8>) -> Result<Vec<u8>> {
    let mut decompressed: Vec<u8> = Vec::new();
    match flate2::read::ZlibDecoder::new(data.as_slice()).read_to_end(&mut decompressed) {
        Ok(_) => Ok(decompressed),
        Err(_) => Err(PngError::DecompressionFailed),
    }
}

fn paeth_predictor(a: i32, b: i32, c: i32) -> i32 {
    let p = a.wrapping_add(b).wrapping_sub(c);
    let pa = p.abs_diff(a);
    let pb = p.abs_diff(b);
    let pc = p.abs_diff(c);
    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}

/// Return the value of the A byte according to the png specification The A byte is
/// defined as the byte to the left of the current byte in the scanline. If we are
/// in the beginning of a scanline the A byte is 0
fn get_a(scanline_idx: usize, bytes_per_scanline: usize, byte_idx: usize, rec: &[u8]) -> i32 {
    if byte_idx > 0 {
        rec[scanline_idx * bytes_per_scanline + byte_idx - 1] as i32
    } else {
        0
    }
}
/// Return the value of the B byte according to the png specification The B byte is
/// defined as the byte "above" the current byte in the scanline, ie the byte at
/// the same position in the scanline from the previous scanline If we are
/// on the first scanline there will be no scanline above and B will be 0
fn get_b(scanline_idx: usize, bytes_per_scanline: usize, byte_idx: usize, rec: &[u8]) -> i32 {
    if scanline_idx > 0 {
        rec[(scanline_idx - 1) * bytes_per_scanline + byte_idx] as i32
    } else {
        0
    }
}
/// Return the value of the C byte according to the png specification The C byte is
/// defined as the byte to the left of the B byte. If we are on the first scanline
/// or the first byte in a scanline C will be 0
fn get_c(scanline_idx: usize, bytes_per_scanline: usize, byte_idx: usize, rec: &[u8]) -> i32 {
    if scanline_idx > 0 && byte_idx > 0 {
        rec[(scanline_idx - 1) * bytes_per_scanline + byte_idx - 1] as i32
    } else {
        0
    }
}

/// Perform reconstruction on the png image data and return a vector containing
/// the decoded data
fn reconstruct(data: &[u8], width: u32, height: u32, bit_depth: u8) -> Result<Vec<u8>> {
    let mut res: Vec<u8> = Vec::new();
    let bits_per_scanline = (width * bit_depth as u32) as f32;

    // How many bytes required to store each scanline, excluding the filter
    // byte
    let bytes_per_scanline = (bits_per_scanline / 8.0).ceil() as usize;

    let mut byte_count = 0;
    for scanline_idx in 0..height as usize {
        // let filter_type = data[scanline_idx * bytes_per_scanline];
        let filter_type = data[byte_count];
        byte_count += 1;

        for byte_idx in 0..bytes_per_scanline {
            let x = data[byte_count] as i32;
            byte_count += 1;

            // A bit unessecary to get these each iteration regardless of
            // filter type but it looks a little cleaner code-wise
            let a = get_a(scanline_idx, bytes_per_scanline, byte_idx, &res);
            let b = get_b(scanline_idx, bytes_per_scanline, byte_idx, &res);
            let c = get_c(scanline_idx, bytes_per_scanline, byte_idx, &res);

            let filt_x = match filter_type {
                0 => x,                            // None
                1 => x + a,                        // Sub
                2 => x + b,                        // Up
                3 => x + (a + b) / 2,              // Average
                4 => x + paeth_predictor(a, b, c), // Paeth
                _ => return Err(PngError::FilterNotSupported(filter_type)),
            };
            res.push((filt_x & 0xFF) as u8);
        }
    }

    Ok(res)
}

/// Parse the contents of a .png file pointed to by path and return a PngImage
/// struct containing the parsed png image. Note that this does not include
/// conversion from scanlines to actual RGB values, only decompression and
/// reconstruction
pub fn parse_png(path: &String) -> Result<PngImage> {
    let png_buf = read_file(path)?;
    let chunks = parse_chunks(&png_buf)?;

    // First index should contain an IHDR
    let ihdr_chunk = {
        let chunk = &chunks[0];
        if !matches!(chunk.chunk_type, ChunkType::Ihrd) {
            Err(PngError::WrongFormat(
                "First chunk type != IHDR".to_string(),
            ))
        } else if chunk.chunk_data.is_none() {
            Err(PngError::WrongFormat(
                "IHDR chunk has no chunk data".to_string(),
            ))
        } else if chunk.chunk_data.as_ref().unwrap().len() != 13 {
            Err(PngError::WrongFormat("IHDR chunk len != 13".to_string()))
        } else {
            Ok(chunk)
        }
    }?;

    // Parse the metadata from IHDR
    let ihdr_data = ihdr_chunk.chunk_data.as_ref().unwrap();

    let width = u32::from_be_bytes(ihdr_data[0..4].try_into().unwrap());
    let height = u32::from_be_bytes(ihdr_data[4..8].try_into().unwrap());
    let bit_depth = ihdr_data[8];
    let color_type = match ihdr_data[9] {
        0 => Ok(ColorType::Grayscale),
        2 => Ok(ColorType::Truecolor),
        3 => Ok(ColorType::IndexedColor),
        4 => Ok(ColorType::GrayScaleWithAlpha),
        6 => Ok(ColorType::TrueColorWithAlpha),
        _ => Err(PngError::WrongFormat("Invalid color type".to_string())),
    }?;

    let compression_method = match ihdr_data[10] {
        0 => Ok(CompressionMethod::DeflateInflate),
        _ => Err(PngError::WrongFormat(
            "Invalid compression method".to_string(),
        )),
    }?;

    let filter_method = match ihdr_data[11] {
        0 => Ok(FilterMethod::FiveTypeAdaptive),
        _ => Err(PngError::WrongFormat("Invalid filter method".to_string())),
    }?;

    let interlace_method = match ihdr_data[12] {
        0 => Ok(InterlaceMethod::NoInterlace),
        1 => Ok(InterlaceMethod::Adam7Interlace),
        _ => Err(PngError::WrongFormat(
            "Invalid interlace method".to_string(),
        )),
    }?;

    // We do not support interlacing
    if matches!(interlace_method, InterlaceMethod::Adam7Interlace) {
        return Err(PngError::NotSupported("Adam7 interlacing".to_string()));
    }

    // We only support index-colored images
    if !matches!(color_type, ColorType::IndexedColor) {
        return Err(PngError::NotSupported(
            "Only indexed color images are supported".to_string(),
        ));
    }

    let palette = match parse_palette(&chunks) {
        Some(palette) => Some(palette),
        None => {
            return Err(PngError::WrongFormat(
                "PLTE chunk missing, this should always be present in index \
                colored images"
                    .to_string(),
            ))
        }
    };

    // Collect data from all IDAT blocks into a Vec<u8> and perform operations
    // to reconstruct the image data
    let idat_data = collect_idat_data(chunks);
    let decompressed = decompress(&idat_data)?;
    let data = reconstruct(&decompressed, width, height, bit_depth)?;

    Ok(PngImage {
        width,
        height,
        bit_depth,
        color_type,
        compression_method,
        filter_method,
        interlace_method,
        palette,
        data,
    })
}

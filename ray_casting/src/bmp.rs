use std::fs::File;
use std::io::{Write, BufWriter};

const BMP_HEADER_SIZE: usize = 54;
const BMP_PIXEL_OFFSET: usize = 54;
const BMP_BITS_PER_PIXEL: usize = 24; // Ajustado a 24 bits para RGB

pub fn write_bmp_file(
    file_path: &str,
    buffer: &[u32],
    width: usize,
    height: usize,
) -> std::io::Result<()> {
    let file = File::create(file_path)?;
    let mut writer = BufWriter::new(file);

    write_bmp_header(&mut writer, width, height)?;
    write_pixel_data(&mut writer, buffer, width, height)?;

    Ok(())
}

fn write_bmp_header(
    file: &mut BufWriter<File>,
    width: usize,
    height: usize,
) -> std::io::Result<()> {
    let file_size = BMP_HEADER_SIZE + (width * height * 3); // 3 bytes por píxel para 24 bits
    let pixel_data_size = width * height * 3;

    // BMP Signature
    file.write_all(b"BM")?;

    // File size
    file.write_all(&(file_size as u32).to_le_bytes())?;

    // Reserved
    file.write_all(&[0; 4])?;

    // Pixel data offset
    file.write_all(&(BMP_PIXEL_OFFSET as u32).to_le_bytes())?;

    // Header size
    file.write_all(&(40 as u32).to_le_bytes())?;

    // Image width
    file.write_all(&(width as u32).to_le_bytes())?;

    // Image height
    file.write_all(&(height as u32).to_le_bytes())?;

    // Color planes
    file.write_all(&(1 as u16).to_le_bytes())?;

    // Bits per pixel
    file.write_all(&(BMP_BITS_PER_PIXEL as u16).to_le_bytes())?;

    // Compression method (no compression)
    file.write_all(&(0 as u32).to_le_bytes())?;

    // Image size (can be 0 for uncompressed images)
    file.write_all(&(pixel_data_size as u32).to_le_bytes())?;

    // Horizontal resolution (pixels per meter)
    file.write_all(&(2835 as u32).to_le_bytes())?; // 72 DPI

    // Vertical resolution (pixels per meter)
    file.write_all(&(2835 as u32).to_le_bytes())?; // 72 DPI

    // Number of colors (0 means default)
    file.write_all(&(0 as u32).to_le_bytes())?;

    // Important colors (0 means all are important)
    file.write_all(&(0 as u32).to_le_bytes())?;

    Ok(())
}

fn write_pixel_data(
    file: &mut BufWriter<File>,
    buffer: &[u32],
    width: usize,
    height: usize,
) -> std::io::Result<()> {
    let row_size = (BMP_BITS_PER_PIXEL / 8) * width;
    let padding_size = (4 - (row_size % 4)) % 4;

    for y in (0..height).rev() {
        let row_start = y * width;
        let row_end = row_start + width;

        for &pixel in &buffer[row_start..row_end] {
            let b = (pixel & 0xFF) as u8;
            let g = ((pixel >> 8) & 0xFF) as u8;
            let r = ((pixel >> 16) & 0xFF) as u8;

            file.write_all(&[b, g, r])?;
        }

        // Write padding bytes
        if padding_size > 0 {
            file.write_all(&vec![0; padding_size])?;
        }
    }

    Ok(())
}

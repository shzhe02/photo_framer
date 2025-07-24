use std::path::PathBuf;

use image::{
    GenericImageView, ImageError, ImageReader, Rgb, RgbImage,
    imageops::{FilterType, overlay},
};

#[derive(Clone, Copy)]
pub enum Sizing {
    Dimensions(u32, u32),
    AspectRatio(f32, f32),
}

pub fn frame_image(input: &PathBuf, output: &PathBuf, sizing: Sizing) -> Result<(), ImageError> {
    let mut img = ImageReader::open(input)?.decode()?;
    let mut dim = img.dimensions();
    let mut background_image = match sizing {
        Sizing::Dimensions(w, h) => {
            img = img.resize(w, h, FilterType::Lanczos3);
            dim = img.dimensions();
            RgbImage::from_pixel(w, h, Rgb([255, 255, 255]))
        }
        Sizing::AspectRatio(w, h) => {
            if (dim.0 as f32 / w) < dim.1 as f32 / h {
                // Border bars are vertical
                RgbImage::from_pixel((dim.1 as f32 * (w / h)) as u32, dim.1, Rgb([255, 255, 255]))
            } else {
                // Border bars are horizontal
                RgbImage::from_pixel(dim.0, (dim.0 as f32 * (h / w)) as u32, Rgb([255, 255, 255]))
            }
        }
    };
    let background_dim = background_image.dimensions();
    // Widths are the same => Horizontal bars
    if dim.0 == background_dim.0 {
        let offset = (background_dim.1 - dim.1) / 2;
        overlay(&mut background_image, &img.to_rgb8(), 0, offset as i64);
    } else {
        // Heights are the same => Vertical bars
        let offset = (background_dim.0 - dim.0) / 2;
        overlay(&mut background_image, &img.to_rgb8(), offset as i64, 0);
    }
    background_image.save(output)?;
    Ok(())
}

use std::hash::{DefaultHasher, Hash, Hasher};

use image::{ImageBuffer, Pixel};

use crate::{tile::vis::DefaultPixel, GridPos2D};

use self::error::VisError;

pub mod collection;
pub mod error;
pub mod ops;

#[derive(Debug, Clone, Copy)]
pub(crate) struct EmptyTile<P, const WIDTH: usize, const HEIGHT: usize>
where
    P: Pixel,
{
    tile_id: u64,
    pixels: [[P; WIDTH]; HEIGHT],
}

impl<P, const WIDTH: usize, const HEIGHT: usize> EmptyTile<P, WIDTH, HEIGHT>
where
    P: DefaultPixel + Hash,
{
    pub fn new(pixels: [[P; WIDTH]; HEIGHT]) -> Self {
        let mut hasher = DefaultHasher::default();
        pixels.hash(&mut hasher);
        let tile_id = hasher.finish();

        Self { tile_id, pixels }
    }
}

/// Writes pixels array into image buffer at provided [GridPos2D].
pub(crate) fn write_tile<P, const WIDTH: usize, const HEIGHT: usize>(
    image_buffer: &mut ImageBuffer<P, Vec<P::Subpixel>>,
    pos: GridPos2D,
    pixels: &[[P; WIDTH]; HEIGHT],
) -> Result<(), VisError<WIDTH, HEIGHT>>
where
    P: Pixel,
{
    let (mut x_pos, mut y_pos) = pos;
    x_pos *= WIDTH as u32;
    y_pos *= HEIGHT as u32;

    for (y, row) in pixels.iter().enumerate() {
        for (x, pixel) in row.iter().enumerate() {
            if let Some(img_pix) =
                image_buffer.get_pixel_mut_checked(x_pos + x as u32, y_pos + y as u32)
            {
                *img_pix = *pixel;
            } else {
                return Err(VisError::new_io(
                    false,
                    pos,
                    (x_pos + x as u32, y_pos + y as u32),
                ));
            }
        }
    }
    Ok(())
}

/// Reads pixels from image that represents the tile at specified [GridPos2D].
pub(crate) fn read_tile<P, const WIDTH: usize, const HEIGHT: usize>(
    pixels: &mut [[P; WIDTH]; HEIGHT],
    image_buffer: &ImageBuffer<P, Vec<P::Subpixel>>,
    pos: GridPos2D,
) -> Result<(), VisError<WIDTH, HEIGHT>>
where
    P: Pixel,
{
    let (mut x_pos, mut y_pos) = pos;
    x_pos *= WIDTH as u32;
    y_pos *= HEIGHT as u32;

    for (y, row) in pixels.iter_mut().enumerate() {
        for (x, pixel) in row.iter_mut().enumerate() {
            if let Some(tile_pix) =
                image_buffer.get_pixel_checked(x_pos + x as u32, y_pos + y as u32)
            {
                *pixel = *tile_pix;
            } else {
                return Err(VisError::new_io(
                    true,
                    pos,
                    (x_pos + x as u32, y_pos + y as u32),
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use image::{ImageBuffer, Rgb};

    use crate::tile::vis::{DefaultPixel, DefaultVisPixel};

    use super::{read_tile, write_tile};

    const PIXELS: [DefaultVisPixel; 4] = [
        Rgb::<u8>([152, 152, 152]),
        Rgb::<u8>([50, 50, 152]),
        Rgb::<u8>([152, 50, 50]),
        Rgb::<u8>([152, 152, 50]),
    ];

    const PIX_ARRAYS: [[[DefaultVisPixel; 2]; 2]; 4] = [
        [[PIXELS[0], PIXELS[1]], [PIXELS[2], PIXELS[3]]],
        [[PIXELS[2], PIXELS[1]], [PIXELS[2], PIXELS[1]]],
        [[PIXELS[3], PIXELS[2]], [PIXELS[1], PIXELS[3]]],
        [[PIXELS[1], PIXELS[2]], [PIXELS[1], PIXELS[3]]],
    ];

    #[test]
    fn can_write_pixels() {
        let mut image = ImageBuffer::new(4, 4);

        let positions = [(0, 0), (0, 1), (1, 0), (1, 1)];

        for i_arr in 0..PIX_ARRAYS.len() {
            write_tile(&mut image, positions[i_arr], &PIX_ARRAYS[i_arr]).unwrap();
        }

        assert_eq!(16, image.pixels().len());
    }

    #[test]
    fn written_and_read_identical() {
        let mut image = ImageBuffer::new(4, 4);

        let positions = [(0, 0), (0, 1), (1, 0), (1, 1)];

        for i_arr in 0..PIX_ARRAYS.len() {
            write_tile(&mut image, positions[i_arr], &PIX_ARRAYS[i_arr]).unwrap();
        }

        for i_arr in 0..PIX_ARRAYS.len() {
            let mut pixels = [[DefaultVisPixel::pix_default(); 2]; 2];
            read_tile(&mut pixels, &image, positions[i_arr]).unwrap();

            assert_eq!(PIX_ARRAYS[i_arr], pixels);
        }
    }
}

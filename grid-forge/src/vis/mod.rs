//! Allows operating on image representations of [`GridMap2D`](crate::map::GridMap2D)

use std::hash::{DefaultHasher, Hash, Hasher};

use image::{ImageBuffer, Luma, LumaA, Pixel, Rgb, Rgba};

use crate::tile::{GridPosition, GridTile, GridTileRef, GridTileRefMut, TileContainer, TileData};

use self::error::VisError;

pub mod collection;
pub mod error;
pub mod ops;

/// Visual representation of tile which is empty.
#[derive(Debug, Clone, Copy)]
pub(crate) struct EmptyTile<P, const WIDTH: usize, const HEIGHT: usize>
where
    P: Pixel,
{
    tile_id: u64,
    pixels: [[P; WIDTH]; HEIGHT],
}

impl<P: Pixel, const WIDTH: usize, const HEIGHT: usize> TileData for EmptyTile<P, WIDTH, HEIGHT> {}

impl<P, const WIDTH: usize, const HEIGHT: usize> EmptyTile<P, WIDTH, HEIGHT>
where
    P: PixelWithDefault + Hash,
{
    pub fn new(pixels: [[P; WIDTH]; HEIGHT]) -> Self {
        let mut hasher = DefaultHasher::default();
        pixels.hash(&mut hasher);
        let tile_id = hasher.finish();

        Self { tile_id, pixels }
    }
}

/// Writes pixels array into image buffer at provided [GridPosition].
pub fn write_tile<P, const WIDTH: usize, const HEIGHT: usize>(
    image_buffer: &mut ImageBuffer<P, Vec<P::Subpixel>>,
    pos: GridPosition,
    pixels: &[[P; WIDTH]; HEIGHT],
) -> Result<(), VisError<WIDTH, HEIGHT>>
where
    P: Pixel,
{
    let (mut x_pos, mut y_pos) = pos.xy();
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

/// Reads pixels from image that represents the tile at specified [`GridPosition`].
pub fn read_tile<P, const WIDTH: usize, const HEIGHT: usize>(
    pixels: &mut [[P; WIDTH]; HEIGHT],
    image_buffer: &ImageBuffer<P, Vec<P::Subpixel>>,
    pos: &GridPosition,
) -> Result<(), VisError<WIDTH, HEIGHT>>
where
    P: Pixel,
{
    let (mut x_pos, mut y_pos) = pos.xy();
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
                    *pos,
                    (x_pos + x as u32, y_pos + y as u32),
                ));
            }
        }
    }
    Ok(())
}

/// Default pixel type used by `grid_forge`.
pub type DefaultVisPixel = Rgb<u8>;

/// Trait for retrieving default value for pixels, necessary for [`VisCollection`](crate::vis::collection::VisCollection).
///
/// Derived for common pixel types.
pub trait PixelWithDefault
where
    Self: Pixel + Hash + PartialEq,
{
    fn pix_default() -> Self;
}

impl<T> PixelWithDefault for Rgb<T>
where
    T: image::Primitive,
    Self: Pixel + Hash + PartialEq,
{
    fn pix_default() -> Self {
        Rgb([T::DEFAULT_MIN_VALUE; 3])
    }
}

impl<T> PixelWithDefault for Rgba<T>
where
    T: image::Primitive,
    Self: Pixel + Hash + PartialEq,
{
    fn pix_default() -> Self {
        Rgba([T::DEFAULT_MIN_VALUE; 4])
    }
}

impl<T> PixelWithDefault for Luma<T>
where
    T: image::Primitive,
    Self: Pixel + Hash + PartialEq,
{
    fn pix_default() -> Self {
        Luma([T::DEFAULT_MIN_VALUE])
    }
}

impl<T> PixelWithDefault for LumaA<T>
where
    T: image::Primitive,
    Self: Pixel + Hash + PartialEq,
{
    fn pix_default() -> Self {
        LumaA([T::DEFAULT_MIN_VALUE; 2])
    }
}

/// Trait allowing retrieving pixels directly on basis of [`TileData`].
///
/// While it is possible to keep the pixel data inside the instance of each tile, in these cases it is recommended to
/// use [`VisCollection`](crate::vis::collection::VisCollection). On other hand, it is handy for tile types which pixels
/// can be generated programatically on basis of its data and/or implementing
/// [`IdentifiableTileData`](crate::tile::identifiable::IdentifiableTileData) is for any reason not possible or viable.
pub trait VisTileData<P, const WIDTH: usize, const HEIGHT: usize>
where
    Self: TileData,
    P: Pixel,
{
    fn vis_pixels(&self) -> [[P; WIDTH]; HEIGHT];
}

pub trait VisTile2D<Data, P, const WIDTH: usize, const HEIGHT: usize>
where
    Self: TileContainer + AsRef<Data>,
    Data: VisTileData<P, WIDTH, HEIGHT>,
    P: Pixel,
{
    /// Writes tile pixels to the image buffer, adjusting the target pixels based on the tile [`GridPosition`].
    fn vis_to_buffer(
        &self,
        image_buffer: &mut ImageBuffer<P, Vec<P::Subpixel>>,
    ) -> Result<(), VisError<WIDTH, HEIGHT>>;
}

impl<Data, P, const WIDTH: usize, const HEIGHT: usize> VisTile2D<Data, P, WIDTH, HEIGHT>
    for GridTile<Data>
where
    Data: VisTileData<P, WIDTH, HEIGHT>,
    P: Pixel,
{
    fn vis_to_buffer(
        &self,
        image_buffer: &mut ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>,
    ) -> Result<(), VisError<WIDTH, HEIGHT>> {
        vis_to_buffer(
            self.grid_position(),
            &self.as_ref().vis_pixels(),
            image_buffer,
        )
    }
}

impl<Data, P, const WIDTH: usize, const HEIGHT: usize> VisTile2D<Data, P, WIDTH, HEIGHT>
    for GridTileRef<'_, Data>
where
    Data: VisTileData<P, WIDTH, HEIGHT>,
    P: Pixel,
{
    fn vis_to_buffer(
        &self,
        image_buffer: &mut ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>,
    ) -> Result<(), VisError<WIDTH, HEIGHT>> {
        vis_to_buffer(
            self.grid_position(),
            &self.as_ref().vis_pixels(),
            image_buffer,
        )
    }
}

impl<Data, P, const WIDTH: usize, const HEIGHT: usize> VisTile2D<Data, P, WIDTH, HEIGHT>
    for GridTileRefMut<'_, Data>
where
    Data: VisTileData<P, WIDTH, HEIGHT>,
    P: Pixel,
{
    fn vis_to_buffer(
        &self,
        image_buffer: &mut ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>,
    ) -> Result<(), VisError<WIDTH, HEIGHT>> {
        vis_to_buffer(
            self.grid_position(),
            &self.as_ref().vis_pixels(),
            image_buffer,
        )
    }
}

#[inline]
fn vis_to_buffer<P, const WIDTH: usize, const HEIGHT: usize>(
    position: GridPosition,
    pixels: &[[P; WIDTH]; HEIGHT],
    image_buffer: &mut ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>,
) -> Result<(), VisError<WIDTH, HEIGHT>>
where
    P: Pixel,
{
    let (mut x_pos, mut y_pos) = (*position.x(), *position.y());
    x_pos *= WIDTH as u32;
    y_pos *= HEIGHT as u32;

    for (y, row) in pixels.iter().enumerate() {
        for (x, pixel) in row.iter().enumerate() {
            match image_buffer.get_pixel_mut_checked(x_pos + x as u32, y_pos + y as u32) {
                Some(p) => *p = *pixel,
                None => {
                    return Err(VisError::new_io(
                        false,
                        position,
                        (x_pos + x as u32, y_pos + y as u32),
                    ))
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use image::{ImageBuffer, Pixel, Rgb};

    use crate::{
        tile::{GridPosition, GridTile, TileData},
        vis::PixelWithDefault,
    };

    use super::{read_tile, write_tile, DefaultVisPixel, VisTile2D, VisTileData};

    struct TestTileData {
        pixels: [[DefaultVisPixel; 3]; 3],
    }

    impl TileData for TestTileData {}

    impl TestTileData {
        fn get_test() -> Self {
            TestTileData {
                pixels: [
                    [
                        DefaultVisPixel::from([0, 0, 0]),
                        DefaultVisPixel::from([10, 10, 10]),
                        DefaultVisPixel::from([20, 20, 20]),
                    ],
                    [
                        DefaultVisPixel::from([30, 30, 30]),
                        DefaultVisPixel::from([40, 40, 40]),
                        DefaultVisPixel::from([50, 50, 50]),
                    ],
                    [
                        DefaultVisPixel::from([60, 60, 60]),
                        DefaultVisPixel::from([70, 70, 70]),
                        DefaultVisPixel::from([80, 80, 80]),
                    ],
                ],
            }
        }
    }

    impl VisTileData<DefaultVisPixel, 3, 3> for TestTileData {
        fn vis_pixels(&self) -> [[DefaultVisPixel; 3]; 3] {
            self.pixels
        }
    }

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

        let positions = [
            GridPosition::new_xy(0, 0),
            GridPosition::new_xy(0, 1),
            GridPosition::new_xy(1, 0),
            GridPosition::new_xy(1, 1),
        ];

        for i_arr in 0..PIX_ARRAYS.len() {
            write_tile(&mut image, positions[i_arr], &PIX_ARRAYS[i_arr]).unwrap();
        }

        assert_eq!(16, image.pixels().len());
    }

    #[test]
    fn written_and_read_identical() {
        let mut image = ImageBuffer::new(4, 4);

        let positions = [
            GridPosition::new_xy(0, 0),
            GridPosition::new_xy(0, 1),
            GridPosition::new_xy(1, 0),
            GridPosition::new_xy(1, 1),
        ];

        for i_arr in 0..PIX_ARRAYS.len() {
            write_tile(&mut image, positions[i_arr], &PIX_ARRAYS[i_arr]).unwrap();
        }

        for i_arr in 0..PIX_ARRAYS.len() {
            let mut pixels = [[DefaultVisPixel::pix_default(); 2]; 2];
            read_tile(&mut pixels, &image, &positions[i_arr]).unwrap();

            assert_eq!(PIX_ARRAYS[i_arr], pixels);
        }
    }

    #[test]
    fn buffer_same_as_pix() {
        let tile = GridTile::new(GridPosition::new_xy(0, 0), TestTileData::get_test());

        let mut buffer = ImageBuffer::new(3, 3);
        tile.vis_to_buffer(&mut buffer).unwrap();

        for y in 0..3 {
            for x in 0..3 {
                assert_eq!(
                    tile.as_ref().vis_pixels()[y][x],
                    buffer.get_pixel(x as u32, y as u32).to_rgb()
                );
            }
        }
    }
}

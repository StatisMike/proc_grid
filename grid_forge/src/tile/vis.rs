use std::hash::Hash;

use image::{ImageBuffer, Luma, LumaA, Pixel, Rgb, Rgba};

use super::{GridPosition, GridTile, GridTileRef, GridTileRefMut, TileData, WithTilePosition};

pub type DefaultVisPixel = Rgb<u8>;

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

pub trait VisTileData<P, const WIDTH: usize, const HEIGHT: usize>
where
    Self: TileData,
    P: Pixel,
{
    fn vis_pixels(&self) -> [[P; WIDTH]; HEIGHT];
}

pub trait VisTile2D<Data, P, const WIDTH: usize, const HEIGHT: usize>
where
    Self: WithTilePosition,
    Data: VisTileData<P, WIDTH, HEIGHT>,
    P: Pixel,
{
    fn vis_pixels(&self) -> [[P; WIDTH]; HEIGHT];

    fn vis_to_buffer(&self, image_buffer: &mut ImageBuffer<P, Vec<P::Subpixel>>);
}

impl<Data, P, const WIDTH: usize, const HEIGHT: usize> VisTile2D<Data, P, WIDTH, HEIGHT>
    for GridTile<Data>
where
    Data: VisTileData<P, WIDTH, HEIGHT>,
    P: Pixel,
{
    fn vis_pixels(&self) -> [[P; WIDTH]; HEIGHT] {
        self.inner().vis_pixels()
    }

    fn vis_to_buffer(&self, image_buffer: &mut ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>) {
        vis_to_buffer(self.position, &self.vis_pixels(), image_buffer);
    }
}

impl<Data, P, const WIDTH: usize, const HEIGHT: usize> VisTile2D<Data, P, WIDTH, HEIGHT>
    for GridTileRef<'_, Data>
where
    Data: VisTileData<P, WIDTH, HEIGHT>,
    P: Pixel,
{
    fn vis_pixels(&self) -> [[P; WIDTH]; HEIGHT] {
        self.inner().vis_pixels()
    }

    fn vis_to_buffer(&self, image_buffer: &mut ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>) {
        vis_to_buffer(self.position, &self.vis_pixels(), image_buffer);
    }
}

impl<Data, P, const WIDTH: usize, const HEIGHT: usize> VisTile2D<Data, P, WIDTH, HEIGHT>
    for GridTileRefMut<'_, Data>
where
    Data: VisTileData<P, WIDTH, HEIGHT>,
    P: Pixel,
{
    fn vis_pixels(&self) -> [[P; WIDTH]; HEIGHT] {
        self.inner().vis_pixels()
    }

    fn vis_to_buffer(&self, image_buffer: &mut ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>) {
        vis_to_buffer(self.position, &self.vis_pixels(), image_buffer);
    }
}

#[inline]
fn vis_to_buffer<P, const WIDTH: usize, const HEIGHT: usize>(
    position: GridPosition,
    pixels: &[[P; WIDTH]; HEIGHT],
    image_buffer: &mut ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>,
) where
    P: Pixel,
{
    let (mut x_pos, mut y_pos) = (*position.x(), *position.y());
    x_pos *= WIDTH as u32;
    y_pos *= HEIGHT as u32;

    for (y, row) in pixels.iter().enumerate() {
        for (x, pixel) in row.iter().enumerate() {
            image_buffer.put_pixel(x_pos + x as u32, y_pos + y as u32, *pixel)
        }
    }
}

#[cfg(test)]
mod tests {
    use image::{ImageBuffer, Pixel};

    use crate::tile::{GridPosition, GridTile, TileData};

    use super::{DefaultVisPixel, VisTile2D, VisTileData};

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

    #[test]
    fn buffer_same_as_pix() {
        let tile = GridTile::new(GridPosition::new_xy(0, 0), TestTileData::get_test());

        let mut buffer = ImageBuffer::new(3, 3);
        tile.vis_to_buffer(&mut buffer);

        for y in 0..3 {
            for x in 0..3 {
                assert_eq!(
                    tile.vis_pixels()[y][x],
                    buffer.get_pixel(x as u32, y as u32).to_rgb()
                );
            }
        }
    }
}

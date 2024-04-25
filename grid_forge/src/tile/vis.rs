use image::{ImageBuffer, Luma, LumaA, Pixel, Rgb, Rgba};

use super::GridTile2D;

pub type DefaultVisPixel = Rgb<u8>;

pub trait DefaultPixel {
    fn pix_def() -> Self;
}

impl<T> DefaultPixel for Rgb<T>
where
    T: image::Primitive,
{
    fn pix_def() -> Self {
        Rgb([T::DEFAULT_MIN_VALUE; 3])
    }
}

impl<T> DefaultPixel for Rgba<T>
where
    T: image::Primitive,
{
    fn pix_def() -> Self {
        Rgba([T::DEFAULT_MIN_VALUE; 4])
    }
}

impl<T> DefaultPixel for Luma<T>
where
    T: image::Primitive,
{
    fn pix_def() -> Self {
        Luma([T::DEFAULT_MIN_VALUE])
    }
}

impl<T> DefaultPixel for LumaA<T>
where
    T: image::Primitive,
{
    fn pix_def() -> Self {
        LumaA([T::DEFAULT_MIN_VALUE; 2])
    }
}

pub trait VisTile2D<P, const WIDTH: usize, const HEIGHT: usize>
where
    Self: GridTile2D + Sized,
    P: Pixel,
{
    fn vis_pixels(&self) -> [[P; WIDTH]; HEIGHT];

    fn vis_to_buffer(&self, image_buffer: &mut ImageBuffer<P, Vec<P::Subpixel>>) {
        let (mut x_pos, mut y_pos) = self.grid_position();
        x_pos *= WIDTH as u32;
        y_pos *= HEIGHT as u32;

        for (y, row) in self.vis_pixels().iter().enumerate() {
            for (x, pixel) in row.iter().enumerate() {
                image_buffer.put_pixel(x_pos + x as u32, y_pos + y as u32, *pixel)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use image::{GenericImageView, ImageBuffer, Pixel};

    use crate::{tile::GridTile2D, GridPos2D};

    use super::{DefaultVisPixel, VisTile2D};

    struct TestTile {
        pos: GridPos2D,
        pixels: [[DefaultVisPixel; 3]; 3],
    }

    impl TestTile {
        fn get_test() -> Self {
            TestTile {
                pos: (0, 0),
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

    impl GridTile2D for TestTile {
        fn grid_position(&self) -> GridPos2D {
            self.pos
        }

        fn set_grid_position(&mut self, position: GridPos2D) {
            self.pos = position;
        }
    }

    impl VisTile2D<DefaultVisPixel, 3, 3> for TestTile {
        fn vis_pixels(&self) -> [[DefaultVisPixel; 3]; 3] {
            self.pixels
        }
    }

    #[test]
    fn buffer_same_as_pix() {
        let tile = TestTile::get_test();

        let mut buffer = ImageBuffer::new(3, 3);
        tile.vis_to_buffer(&mut buffer);

        for y in 0..3 {
            for x in 0..3 {
                assert_eq!(
                    tile.pixels[y][x],
                    buffer.get_pixel(x as u32, y as u32).to_rgb()
                );
            }
        }
    }
}

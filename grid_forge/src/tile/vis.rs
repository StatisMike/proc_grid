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

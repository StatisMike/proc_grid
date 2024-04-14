use image::{ImageBuffer, Pixel};

use super::GridTile2D;

pub trait VisTile2D<P>
where
    Self: GridTile2D + Sized,
    P: Pixel + 'static,
{
    const PIXEL_SIZE: [u32; 2];

    fn vis_pixel(&self) -> P;

    fn vis_to_buffer(&self, image_buffer: &mut ImageBuffer<P, Vec<P::Subpixel>>) {
        let (mut x_pos, mut y_pos) = self.grid_position();
        x_pos *= Self::PIXEL_SIZE[0];
        y_pos *= Self::PIXEL_SIZE[1];

        let pixel = self.vis_pixel();

        for i_x in 0..Self::PIXEL_SIZE[0] {
            for i_y in 0..Self::PIXEL_SIZE[1] {
                image_buffer.put_pixel(x_pos + i_x, y_pos + i_y, pixel);
            }
        }
    }
}

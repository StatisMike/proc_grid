use image::{ImageBuffer, Pixel};

use crate::tile::{vis::VisTile2D, GridTile2D};

use super::GridMap2D;

pub trait VisGrid2D<T, P>
where
    T: GridTile2D + VisTile2D<P>,
    P: Pixel + 'static,
{
    fn vis_grid_map(&self) -> ImageBuffer<P, Vec<P::Subpixel>>;
}

impl<T, P> VisGrid2D<T, P> for GridMap2D<T>
where
    T: GridTile2D + VisTile2D<P>,
    P: Pixel + 'static,
{
    fn vis_grid_map(&self) -> ImageBuffer<P, Vec<<P as Pixel>::Subpixel>> {
        let mut image = ImageBuffer::<P, Vec<P::Subpixel>>::new(
            self.size().x() * T::PIXEL_SIZE[0],
            self.size().y() * T::PIXEL_SIZE[1],
        );
        for pos in self.get_all_positions() {
            self.get_tile_at_position(&pos)
                .unwrap()
                .vis_to_buffer(&mut image);
        }
        image
    }
}

use image::{ImageBuffer, Pixel};

use crate::{
    gen::wfc::WFCTile,
    tile::{vis::VisTile2D, GridTile2D},
};

use super::GridMap2D;

pub trait VisGrid2D<T, P, const WIDTH: usize, const HEIGHT: usize>
where
    T: GridTile2D + VisTile2D<P, WIDTH, HEIGHT>,
    P: Pixel + 'static,
{
    fn vis_grid_map(&self) -> ImageBuffer<P, Vec<P::Subpixel>>;
}

impl<T, P, const WIDTH: usize, const HEIGHT: usize> VisGrid2D<T, P, WIDTH, HEIGHT> for GridMap2D<T>
where
    T: GridTile2D + VisTile2D<P, WIDTH, HEIGHT>,
    P: Pixel + 'static,
{
    fn vis_grid_map(&self) -> ImageBuffer<P, Vec<<P as Pixel>::Subpixel>> {
        let mut image = ImageBuffer::<P, Vec<P::Subpixel>>::new(
            self.size().x() * WIDTH as u32,
            self.size().y() * HEIGHT as u32,
        );
        for pos in self.get_all_positions() {
            self.get_tile_at_position(&pos)
                .unwrap()
                .vis_to_buffer(&mut image);
        }
        image
    }
}

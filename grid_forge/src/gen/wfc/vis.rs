use std::{
    fmt::Display,
    hash::{DefaultHasher, Hash, Hasher},
};

use image::{ImageBuffer, Pixel};

use crate::{
    map::{GridMap2D, GridSize},
    tile::{
        vis::{DefaultPixel, VisTile2D},
        GridTile2D, identifiable::IdentifiableTile,
    },
    GridPos2D,
};

#[derive(Clone, Copy, Debug)]
pub struct WFCVisTile<P, const WIDTH: usize, const HEIGHT: usize>
where
    P: Pixel,
{
    pos: GridPos2D,
    pixels: [[P; WIDTH]; HEIGHT],
    tile_id: Option<u64>,
}

impl<P, const WIDTH: usize, const HEIGHT: usize> GridTile2D for WFCVisTile<P, WIDTH, HEIGHT>
where
    P: Pixel,
{
    fn grid_position(&self) -> GridPos2D {
        self.pos
    }

    fn set_grid_position(&mut self, position: GridPos2D) {
        self.pos = position;
    }
}

impl<P, const WIDTH: usize, const HEIGHT: usize> IdentifiableTile for WFCVisTile<P, WIDTH, HEIGHT>
where
    P: Pixel,
{
    fn get_tile_id(&self) -> u64 {
        self.tile_id
            .expect("can't access `tile_id` before initialization")
    }
}

impl<P, const WIDTH: usize, const HEIGHT: usize> VisTile2D<P, WIDTH, HEIGHT>
    for WFCVisTile<P, WIDTH, HEIGHT>
where
    P: Pixel,
{
    fn vis_pixels(&self) -> [[P; WIDTH]; HEIGHT] {
        self.pixels
    }
}

impl<P, const WIDTH: usize, const HEIGHT: usize> WFCVisTile<P, WIDTH, HEIGHT>
where
    P: Pixel + DefaultPixel + Hash + 'static,
{
    pub fn from_image(
        image: &ImageBuffer<P, Vec<P::Subpixel>>,
    ) -> Result<Self, VisAnalyzeError<WIDTH, HEIGHT>> {
        check_tile_vis_size(image)?;

        let mut pixels = [[P::pix_default(); WIDTH]; HEIGHT];

        for (y, row) in pixels.iter_mut().enumerate() {
            for (x, pix) in row.iter_mut().enumerate() {
                *pix = *image.get_pixel(x as u32, y as u32);
            }
        }

        let wfc_id = Some(Self::calc_wfc_id(pixels));

        Ok(Self {
            pos: (0, 0),
            pixels,
            tile_id: wfc_id,
        })
    }

    fn from_image_for_pos(
        image: &ImageBuffer<P, Vec<P::Subpixel>>,
        pos: GridPos2D,
    ) -> Result<Self, VisAnalyzeError<WIDTH, HEIGHT>> {
        let (init_x, init_y) = (pos.0 * WIDTH as u32, pos.1 * HEIGHT as u32);

        let mut pixels = [[P::pix_default(); WIDTH]; HEIGHT];

        for (y_pix, row) in pixels.iter_mut().enumerate() {
            for (x_pix, pix) in row.iter_mut().enumerate() {
                *pix = *image.get_pixel(init_x + x_pix as u32, init_y + y_pix as u32);
            }
        }

        let wfc_id = Some(Self::calc_wfc_id(pixels));

        Ok(Self {
            pos,
            pixels,
            tile_id: wfc_id,
        })
    }

    pub(crate) fn calc_wfc_id(pixels: [[P; WIDTH]; HEIGHT]) -> u64 {
        let mut hasher = DefaultHasher::default();
        pixels.hash(&mut hasher);
        hasher.finish()
    }
}

pub trait WFCVisGrid2D<T, P, const WIDTH: usize, const HEIGHT: usize>
where
    Self: Sized,
    T: GridTile2D + VisTile2D<P, WIDTH, HEIGHT> + IdentifiableTile,
    P: Pixel + 'static,
{
    fn from_image(
        image: &ImageBuffer<P, Vec<P::Subpixel>>,
    ) -> Result<Self, VisAnalyzeError<WIDTH, HEIGHT>>;
}

impl<P, const WIDTH: usize, const HEIGHT: usize>
    WFCVisGrid2D<WFCVisTile<P, WIDTH, HEIGHT>, P, WIDTH, HEIGHT>
    for GridMap2D<WFCVisTile<P, WIDTH, HEIGHT>>
where
    P: Pixel + DefaultPixel + Hash + 'static,
{
    fn from_image(
        image: &ImageBuffer<P, Vec<P::Subpixel>>,
    ) -> Result<Self, VisAnalyzeError<WIDTH, HEIGHT>> {
        let grid_size = check_grid_vis_size(image)?;
        let mut grid = GridMap2D::new(grid_size);

        for position in grid_size.get_all_possible_positions() {
            grid.insert_tile(WFCVisTile::<P, WIDTH, HEIGHT>::from_image_for_pos(
                image, position,
            )?);
        }

        Ok(grid)
    }
}

// impl<T> WFCResolver<T>
// where
//     T: IdentifiableTile,
// {
//     pub fn get_image(&self)
//     {
//         self.wfc_grid
//     }
// }

//--- Helper functions ---//
fn check_tile_vis_size<P: Pixel + 'static, const WIDTH: usize, const HEIGHT: usize>(
    image: &ImageBuffer<P, Vec<P::Subpixel>>,
) -> Result<(), VisAnalyzeError<WIDTH, HEIGHT>> {
    if image.height() as usize != HEIGHT || image.width() as usize != WIDTH {
        Err(VisAnalyzeError::new_tile(image.width(), image.height()))
    } else {
        Ok(())
    }
}

fn check_grid_vis_size<P: Pixel + 'static, const WIDTH: usize, const HEIGHT: usize>(
    image: &ImageBuffer<P, Vec<P::Subpixel>>,
) -> Result<GridSize, VisAnalyzeError<WIDTH, HEIGHT>> {
    if image.height() as usize % HEIGHT != 0 || image.width() as usize % WIDTH != 0 {
        Err(VisAnalyzeError::new_grid(image.width(), image.height()))
    } else {
        Ok(GridSize::new(
            image.width() / WIDTH as u32,
            image.height() / HEIGHT as u32,
        ))
    }
}

//-------- Errors --------//

#[derive(Debug, Clone)]
pub struct VisAnalyzeError<const WIDTH: usize, const HEIGHT: usize> {
    kind: VisAnalyzeErrorKind,
}

impl<const WIDTH: usize, const HEIGHT: usize> VisAnalyzeError<WIDTH, HEIGHT> {
    fn new_tile(x: u32, y: u32) -> Self {
        Self {
            kind: VisAnalyzeErrorKind::WrongSizeTile { x, y },
        }
    }

    fn new_grid(x: u32, y: u32) -> Self {
        Self {
            kind: VisAnalyzeErrorKind::WrongSizeGrid { x, y },
        }
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> Display for VisAnalyzeError<WIDTH, HEIGHT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            VisAnalyzeErrorKind::WrongSizeTile { x, y } => {
                write!(f, "expected tile pixel size (x: {WIDTH}; y: {HEIGHT}) is incompatible with actual image size: (x: {x}, y: {y})")
            }
            VisAnalyzeErrorKind::WrongSizeGrid { x, y } => {
                write!(f, "extected tile pixel size (x: {WIDTH}; y: {HEIGHT}) is incompatible with GridMap image size: (x: {x}, y: {y})")
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum VisAnalyzeErrorKind {
    WrongSizeTile { x: u32, y: u32 },
    WrongSizeGrid { x: u32, y: u32 },
}

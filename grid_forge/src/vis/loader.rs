use std::{fmt::Display, hash::{DefaultHasher, Hash, Hasher}};

use image::{ImageBuffer, Pixel};

use crate::{map::{GridMap2D, GridSize}, tile::{identifiable::{builder::IdentTileBuilder, IdentifiableTile}, vis::DefaultPixel}, GridPos2D};

use super::{collection::VisCollection, read_tile};

pub struct VisIdentifiableLoader<T, B, P, const WIDTH: usize, const HEIGHT: usize> 
where 
  T: IdentifiableTile,
  B: IdentTileBuilder<T>,
  P: DefaultPixel,
{
  collection: VisCollection<T, P, WIDTH, HEIGHT>,
  builder: B
}

impl <T, B, P, const WIDTH: usize, const HEIGHT: usize> VisIdentifiableLoader<T, B, P, WIDTH, HEIGHT>
where 
  T: IdentifiableTile,
  B: IdentTileBuilder<T>,
  P: DefaultPixel + 'static
{
  pub fn new(collection: VisCollection<T, P, WIDTH, HEIGHT>, builder: B) -> Self
  {
    Self { collection, builder }
  }

  pub fn analyze_grid_image(&mut self, image: &ImageBuffer<P, Vec<P::Subpixel>>) -> Result<GridMap2D<T>, VisLoadError<WIDTH, HEIGHT>>
  {
    let size = check_grid_vis_size(image)?;
    let mut grid = GridMap2D::<T>::new(size);

    for position in size.get_all_possible_positions() {
      if let Some(tile) = self.analyze_tile(image, position) {
        grid.insert_tile(tile);
      }
    }

    Ok(grid)
  }

  fn analyze_tile(&mut self, image: &ImageBuffer<P, Vec<P::Subpixel>>, position: GridPos2D) -> Option<T>
  {
    let mut pixels = [[P::pix_default(); WIDTH]; HEIGHT];
    read_tile(&mut pixels, image, position).unwrap();
    let tile = self.builder.create_identifiable_tile(position, Self::get_tile_id_from_pixels(&pixels));
    match self.collection.add_tile_pixels_manual(tile.get_tile_id(), pixels) {
        super::collection::VisCollectionOutcome::Empty => None,
        _ => Some(tile)
    }
  }

  fn get_tile_id_from_pixels(pixels: &[[P; WIDTH]; HEIGHT]) -> u64
  {
    let mut hasher = DefaultHasher::default();
    pixels.hash(&mut hasher);
    hasher.finish()
  }
}


//--- Helper functions ---//
fn check_tile_vis_size<P: Pixel + 'static, const WIDTH: usize, const HEIGHT: usize>(
  image: &ImageBuffer<P, Vec<P::Subpixel>>,
) -> Result<(), VisLoadError<WIDTH, HEIGHT>> {
  if image.height() as usize != HEIGHT || image.width() as usize != WIDTH {
      Err(VisLoadError::new_tile(image.width(), image.height()))
  } else {
      Ok(())
  }
}

fn check_grid_vis_size<P: Pixel + 'static, const WIDTH: usize, const HEIGHT: usize>(
  image: &ImageBuffer<P, Vec<P::Subpixel>>,
) -> Result<GridSize, VisLoadError<WIDTH, HEIGHT>> {
  if image.height() as usize % HEIGHT != 0 || image.width() as usize % WIDTH != 0 {
      Err(VisLoadError::new_grid(image.width(), image.height()))
  } else {
      Ok(GridSize::new(
          image.width() / WIDTH as u32,
          image.height() / HEIGHT as u32,
      ))
  }
}

//-------- Errors --------//

#[derive(Debug, Clone)]
pub struct VisLoadError<const WIDTH: usize, const HEIGHT: usize> {
  kind: VisLoadErrorKind,
}

impl<const WIDTH: usize, const HEIGHT: usize> VisLoadError<WIDTH, HEIGHT> {
  fn new_tile(x: u32, y: u32) -> Self {
      Self {
          kind: VisLoadErrorKind::WrongSizeTile { x, y },
      }
  }

  fn new_grid(x: u32, y: u32) -> Self {
      Self {
          kind: VisLoadErrorKind::WrongSizeGrid { x, y },
      }
  }
}

impl<const WIDTH: usize, const HEIGHT: usize> Display for VisLoadError<WIDTH, HEIGHT> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      match self.kind {
          VisLoadErrorKind::WrongSizeTile { x, y } => {
              write!(f, "expected tile pixel size (x: {WIDTH}; y: {HEIGHT}) is incompatible with actual image size: (x: {x}, y: {y})")
          }
          VisLoadErrorKind::WrongSizeGrid { x, y } => {
              write!(f, "extected tile pixel size (x: {WIDTH}; y: {HEIGHT}) is incompatible with GridMap image size: (x: {x}, y: {y})")
          }
      }
  }
}

#[derive(Debug, Clone, Copy)]
enum VisLoadErrorKind {
  WrongSizeTile { x: u32, y: u32 },
  WrongSizeGrid { x: u32, y: u32 },
}
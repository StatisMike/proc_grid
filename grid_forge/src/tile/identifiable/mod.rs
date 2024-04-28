use crate::GridPos2D;
use self::builder::ConstructableViaIdentifierTile;

use super::GridTile2D;

pub mod builder;

/// Its implementation makes the specific tile identifiable and discernable from other tile instances in regards to tile
/// type. For the generative algorithms using this trait to match and select tiles, general rules of the tile identity
/// when implementing this trait manually should be:
///
/// - its position **should not be ever taken into account**. Tile of these same type could be placed on different positions
/// on the GridMap.
/// - other properties of the tile (such as visual representation) *can* be taken into account depending on your specific
/// needs.
pub trait IdentifiableTile
where
    Self: GridTile2D,
{
    fn get_tile_id(&self) -> u64;
}

/// Basic tile struct that implements [`IdentifiableTile`], holding only the most basic information.
#[derive(Clone, Copy, Debug)]
pub struct BasicIdentifiableTile2D {
  pos: GridPos2D,
  tile_id: u64,
}

impl GridTile2D for BasicIdentifiableTile2D {
    fn grid_position(&self) -> GridPos2D {
        self.pos
    }

    fn set_grid_position(&mut self, position: GridPos2D) {
        self.pos = position
    }
}

impl IdentifiableTile for BasicIdentifiableTile2D {
    fn get_tile_id(&self) -> u64 {
        self.tile_id
    }
}

impl ConstructableViaIdentifierTile for BasicIdentifiableTile2D {
  fn tile_new(pos: GridPos2D, tile_id: u64) -> Self {
      Self { pos, tile_id }
  }
}
use std::hash::Hash;

use crate::GridPos2D;

#[cfg(feature = "vis")]
pub mod vis;

/// Marker trait for data specific to the tile type. It is used to differentiate between different kinds of tiles,
/// (eg. *forest*, *ocean*, *path*).
///
/// Used by default implementations of traits used in procedural generation.
pub trait GridTileData
where
    Self: Hash,
{
}

/// Trait that needs to be implemented for objects contained within the [GridMap2D](crate::grid::GridMap2D)
pub trait GridTile2D {
    fn grid_position(&self) -> GridPos2D;

    fn set_grid_position(&mut self, position: GridPos2D);
}

pub trait WithGridTileData<Data>
where
    Data: GridTileData,
{
    fn tile_data(&self) -> &Data;
}

mod test {}

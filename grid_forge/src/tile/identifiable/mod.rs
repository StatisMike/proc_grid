//! This module provide a way to identify a tile by its `tile_type_id`.
//!
//! As for exact meaning of *Tile Type* it is mainly left to the direct implementation and use-case, eg. sometimes just
//! the *biome* (like grass tile, water type, desert type etc.) will be distinct types, other times more specific
//! identity will be needed. All these details are left for implementor to decide upon.
//!
//! One thing that should never be taken into account is specific position of the tile in the grid, as it would mean every
//! tile on every grid will be of different type.
//!
//! `tile_type_id` is of [u64] type, which makes it easily implementable by hashing some specific properties of the tile
//! present on the struct.

use self::builders::ConstructableViaIdentifierTile;

use super::TileData;

pub mod builders;
pub mod collection;

/// Its implementation makes the specific tile identifiable and discernable from other tile instances in regards to tile
/// type. For the generative algorithms using this trait to match and select tiles, general rules of the tile identity
/// when implementing this trait manually should be:
///
/// - its position **should not be ever taken into account**. Tile of these same type could be placed on different positions
/// on the GridMap.
/// - other properties of the tile (such as visual representation) *can* be taken into account depending on your specific
/// needs.
pub trait IdentifiableTileData
where
    Self: TileData,
{
    fn tile_type_id(&self) -> u64;
}

/// Basic tile struct that implements [`IdentifiableTileData`], holding only the most basic information.
#[derive(Clone, Copy, Debug)]
pub struct BasicIdentTileData {
    tile_type_id: u64,
}

impl TileData for BasicIdentTileData {}

impl IdentifiableTileData for BasicIdentTileData {
    fn tile_type_id(&self) -> u64 {
        self.tile_type_id
    }
}

impl ConstructableViaIdentifierTile for BasicIdentTileData {
    fn tile_new(tile_type_id: u64) -> Self {
        BasicIdentTileData { tile_type_id }
    }
}

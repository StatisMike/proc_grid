use std::error::Error;
use std::fmt::Display;
use std::{collections::BTreeMap, marker::PhantomData};

use crate::tile::identifiable::IdentifiableTile;
use crate::GridPos2D;

#[derive(Debug, Clone)]
pub struct IdentTileCloneBuilder<T>
where
    T: IdentifiableTile + Clone,
{
    tiles: BTreeMap<u64, T>,
}

impl<T> Default for IdentTileCloneBuilder<T>
where
    T: IdentifiableTile + Clone,
{
    fn default() -> Self {
        Self {
            tiles: BTreeMap::new(),
        }
    }
}

impl<T> IdentTileCloneBuilder<T>
where
    T: IdentifiableTile + Clone,
{
    pub fn add_tiles(&mut self, tiles: Vec<&T>, overwrite: bool) {
        for tile in tiles {
            if !overwrite && self.tiles.contains_key(&tile.get_tile_id()) {
                continue;
            }
            self.tiles.insert(tile.get_tile_id(), tile.clone());
        }
    }
}

impl<T> IdentTileBuilder<T> for IdentTileCloneBuilder<T>
where
    T: IdentifiableTile + Clone,
{
    fn create_identifiable_tile(&self, pos: GridPos2D, tile_id: u64) -> T {
        let mut tile = self
            .tiles
            .get(&tile_id)
            .unwrap_or_else(|| panic!("can't get tile with tile id: {tile_id}"))
            .clone();
        tile.set_grid_position(pos);
        tile
    }

    fn check_missing_tile_creators(&self, tile_ids: &[u64]) -> Result<(), TileBuilderError> {
        let missing_ids = tile_ids
            .iter()
            .filter(|tile_id| !self.tiles.contains_key(tile_id))
            .copied()
            .collect::<Vec<_>>();

        if !missing_ids.is_empty() {
            Err(TileBuilderError::new(&missing_ids))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
pub struct IdentTileFunBuilder<T>
where
    T: IdentifiableTile,
{
    funs: BTreeMap<u64, fn(GridPos2D, u64) -> T>,
}

impl<T> Default for IdentTileFunBuilder<T>
where
    T: IdentifiableTile,
{
    fn default() -> Self {
        Self {
            funs: BTreeMap::new(),
        }
    }
}

impl<T> IdentTileBuilder<T> for IdentTileFunBuilder<T>
where
    T: IdentifiableTile,
{
    fn create_identifiable_tile(&self, pos: GridPos2D, tile_id: u64) -> T {
        let fun = self
            .funs
            .get(&tile_id)
            .unwrap_or_else(|| panic!("can't get tile function with `wfc_id`: {tile_id}"));

        fun(pos, tile_id)
    }

    fn check_missing_tile_creators(&self, tile_ids: &[u64]) -> Result<(), TileBuilderError> {
        let missing_ids = tile_ids
            .iter()
            .filter(|tile_id| !self.funs.contains_key(tile_id))
            .copied()
            .collect::<Vec<_>>();

        if !missing_ids.is_empty() {
            Err(TileBuilderError::new(&missing_ids))
        } else {
            Ok(())
        }
    }
}

pub trait ConstructableViaIdentifierTile
where
    Self: IdentifiableTile,
{
    fn tile_new(pos: GridPos2D, wfc_id: u64) -> Self;
}

/// [`IdentTileBuilder`] which creates new tiles with given identifier based on the tile implementation of
/// [`ConstructableViaIdentifierTile`]. No need to add any tile creators.
#[derive(Debug, Clone)]
pub struct IdentTileTraitBuilder<T>
where
    T: ConstructableViaIdentifierTile,
{
    phantom: PhantomData<T>,
}

impl<T> Default for IdentTileTraitBuilder<T>
where
    T: ConstructableViaIdentifierTile,
{
    fn default() -> Self {
        Self {
            phantom: PhantomData::<T>,
        }
    }
}

impl<T> IdentTileBuilder<T> for IdentTileTraitBuilder<T>
where
    T: ConstructableViaIdentifierTile,
{
    fn create_identifiable_tile(&self, pos: GridPos2D, wfc_id: u64) -> T {
        T::tile_new(pos, wfc_id)
    }

    fn check_missing_tile_creators(&self, _tile_ids: &[u64]) -> Result<(), TileBuilderError> {
        Ok(())
    }
}

/// Trait shared by objects, which given the grid position and tile identifier of given [`IdentifiableTile`]-implementing
/// struct can create correct instance of the tile.
pub trait IdentTileBuilder<T>
where
    T: IdentifiableTile,
{
    /// Creates tile with given tile identifier at given grid position.
    fn create_identifiable_tile(&self, pos: GridPos2D, tile_id: u64) -> T;

    /// Returns vector of missing tile creators (if any).
    fn check_missing_tile_creators(&self, tile_ids: &[u64]) -> Result<(), TileBuilderError>;
}

#[derive(Debug, Clone)]
pub struct TileBuilderError {
    tile_ids: Vec<u64>,
}

impl TileBuilderError {
    fn new(tile_ids: &[u64]) -> Self {
        Self {
            tile_ids: Vec::from(tile_ids),
        }
    }

    pub fn get_missing_tile_ids(&self) -> &[u64] {
        &self.tile_ids
    }
}

impl Display for TileBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "missing tile ids from builder: {missing_ids}",
            missing_ids = self
                .tile_ids
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl Error for TileBuilderError {}

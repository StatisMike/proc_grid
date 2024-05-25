use std::collections::BTreeMap;

use crate::{
    map::GridMap2D,
    tile::{identifiable::IdentifiableTileData, GridPosition, GridTile, TileContainer},
};

pub(crate) mod entrophy;
pub(crate) mod position;

pub use entrophy::EntrophyQueue;
pub use position::*;
// pub (crate) use entrophy::*;
// pub (crate) use position::*;

use rand::Rng;

use super::{
    frequency::FrequencyHints,
    tile::{CollapsibleData, CollapsibleTileData},
};

/// Trait shared by objects that handle the selecting algorithm for next tile to collapse within
/// [`CollapsibleResolver`](crate::gen::collapse::CollapsibleResolver)
#[allow(private_bounds)]
pub trait CollapseQueue
where
    Self: Default + ResolverSelector + Sized,
{
    /// Pop next position for collapsing.
    fn get_next_position(&mut self) -> Option<GridPosition>;

    /// Initialize the queue based on provided tiles.
    fn initialize_queue<T: CollapsibleData>(&mut self, tiles: &[GridTile<T>]);

    /// Update internal based on provided tile.
    fn update_queue<Tile, Data>(&mut self, tile: &Tile)
    where
        Tile: TileContainer + AsRef<Data>,
        Data: CollapsibleData;

    /// Checks the current size of the inner queue.
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool;
}

pub(crate) trait ResolverSelector {
    fn populate_inner_grid<R, Data>(
        &mut self,
        rng: &mut R,
        grid: &mut GridMap2D<Data>,
        positions: &[GridPosition],
        options_with_weights: BTreeMap<u64, u32>,
    ) where
        R: Rng,
        Data: CollapsibleData;

    fn needs_update_after_options_change(&self) -> bool {
        false
    }

    fn propagating(&self) -> bool {
        false
    }

    fn in_propagaton_range(&self, _collapsed: &GridPosition, _candidate: &GridPosition) -> bool {
        false
    }
}

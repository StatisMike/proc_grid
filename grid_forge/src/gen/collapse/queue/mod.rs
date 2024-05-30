use std::collections::BTreeMap;

use crate::map::GridMap2D;
use crate::tile::{GridPosition, GridTile, TileContainer};

pub(crate) mod entrophy;
pub(crate) mod position;

pub use entrophy::EntrophyQueue;
pub use position::*;

use rand::Rng;

use super::tile::CollapsibleTileData;

/// Trait shared by objects that handle the selecting algorithm for next tile to collapse within collapse resolvers.
#[allow(private_bounds)]
pub trait CollapseQueue
where
    Self: Default + ResolverSelector + Sized,
{
    /// Pop next position for collapsing.
    fn get_next_position(&mut self) -> Option<GridPosition>;

    /// Initialize the queue based on provided tiles.
    fn initialize_queue<T: CollapsibleTileData>(&mut self, tiles: &[GridTile<T>]);

    /// Update internal based on provided tile.
    fn update_queue<Tile, Data>(&mut self, tile: &Tile)
    where
        Tile: TileContainer + AsRef<Data>,
        Data: CollapsibleTileData;

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
        Data: CollapsibleTileData;

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

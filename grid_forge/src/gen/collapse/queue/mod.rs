use crate::{
    map::GridMap2D,
    tile::{
        identifiable::{IdentifiableTile, IdentifiableTileData},
        GridPosition, GridTile, GridTileRef,
    },
};

pub(crate) mod entrophy;
pub(crate) mod position;

pub use entrophy::EntrophyQueue;
pub use position::*;
// pub (crate) use entrophy::*;
// pub (crate) use position::*;

use rand::Rng;

use super::{frequency::FrequencyHints, tile::CollapsibleTileData};

/// Trait shared by objects that handle the selecting algorithm for next tile to collapse within
/// [`CollapsibleResolver`](crate::gen::collapse::CollapsibleResolver)
#[allow(private_bounds)]
pub trait CollapseQueue
where
    Self: Default + ResolverSelector,
{
    /// Pop next position for collapsing.
    fn get_next_position(&mut self) -> Option<GridPosition>;

    /// Initialize the queue based on provided tiles.
    fn initialize_queue(&mut self, tiles: &[GridTile<CollapsibleTileData>]);

    /// Update internal based on provided tile.
    fn update_queue<Tile>(&mut self, tile: &Tile)
    where
        Tile: IdentifiableTile<CollapsibleTileData> + AsRef<CollapsibleTileData>;

    /// Checks the current size of the inner queue.
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool;
}

pub(crate) trait ResolverSelector {
    fn populate_inner_grid<Data, R>(
        &mut self,
        rng: &mut R,
        grid: &mut GridMap2D<CollapsibleTileData>,
        positions: &[GridPosition],
        frequencies: &FrequencyHints<Data>,
    ) where
        Data: IdentifiableTileData,
        R: Rng;

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

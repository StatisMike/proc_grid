use crate::tile::{GridPosition, GridTile, TileContainer};

pub(crate) mod entrophy;
pub(crate) mod position;
mod propagator;

pub use entrophy::EntrophyQueue;
pub use position::*;
pub use propagator::*;

use super::tile::CollapsibleTileData;

/// Trait shared by objects that handle the selecting algorithm for next tile to collapse within collapse resolvers.
pub trait CollapseQueue
where
    Self: Default + Sized + private::Sealed,
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

pub(crate) mod private {
    use std::collections::BTreeMap;

    use rand::Rng;

    use crate::{
        gen::collapse::{AdjacencyTable, CollapsibleTileData},
        map::GridMap2D,
        tile::GridPosition,
    };

    pub trait Sealed {
        fn populate_inner_grid<R, Data>(
            &mut self,
            rng: &mut R,
            grid: &mut GridMap2D<Data>,
            positions: &[GridPosition],
            adjacency_table: &AdjacencyTable,
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

        fn in_propagaton_range(
            &self,
            _collapsed: &GridPosition,
            _candidate: &GridPosition,
        ) -> bool {
            false
        }
    }
}

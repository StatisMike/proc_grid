use crate::{map::GridMap2D, tile::identifiable::IdentifiableTile, GridPos2D};

pub(crate) mod entrophy;
pub(crate) mod position;

pub use entrophy::EntrophyQueue;
pub use position::*;
// pub (crate) use entrophy::*;
// pub (crate) use position::*;

use rand::Rng;

use super::{frequency::FrequencyHints, tile::CollapsibleTile};

/// Trait shared by objects that handle the selecting algorithm for next tile to collapse within [`Resolver``]
pub trait CollapseQueue
where
    Self: Default + ResolverSelector,
{
    /// Pop next position for collapsing.
    fn get_next_position(&mut self) -> Option<GridPos2D>;

    /// Initialize the queue based on provided tiles.
    fn initialize_queue(&mut self, tiles: &[CollapsibleTile]);

    /// Update internal based on provided tile.
    fn update_queue(&mut self, tile: &CollapsibleTile);

    /// Checks the current size of the inner queue.
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool;
}

pub(crate) trait ResolverSelector {
    fn populate_inner_grid<R: Rng, InputTile: IdentifiableTile>(
        &mut self,
        rng: &mut R,
        grid: &mut GridMap2D<CollapsibleTile>,
        positions: &[GridPos2D],
        frequencies: &FrequencyHints<InputTile>,
    );

    fn needs_update_after_options_change(&self) -> bool {
        false
    }

    fn propagating(&self) -> bool {
        false
    }
}

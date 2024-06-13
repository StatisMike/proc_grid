mod error;
mod grid;
mod option;
pub mod overlap;
mod queue;
pub mod singular;
mod tile;

use std::ops::Index;

// Flattened reexports
pub use error::CollapseError;
pub use grid::{CollapsedGrid, CollapsibleGrid};
pub use queue::*;
pub use tile::*;

use nohash::{IntMap, IntSet};

use crate::{map::GridDir, tile::GridPosition};

#[derive(Clone, Debug, Default)]
pub(crate) struct Adjacencies {
    inner: Vec<IntSet<u64>>,
}

impl Adjacencies {
    pub fn new() -> Self {
        let mut inner = Vec::new();

        for _ in 0..GridDir::ALL_2D.len() {
            inner.push(IntSet::default());
        }

        Self { inner }
    }

    #[inline(always)]
    pub fn add_at_dir(&mut self, direction: GridDir, id: u64) {
        let v = self.inner.get_mut(direction as usize).unwrap();
        v.insert(id);
    }
}

impl Index<GridDir> for Adjacencies {
    type Output = IntSet<u64>;

    fn index(&self, index: GridDir) -> &Self::Output {
        &self.inner[index as usize]
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct AdjacencyTable {
    inner: IntMap<u64, Adjacencies>,
}

impl AdjacencyTable {
    pub(crate) fn insert_adjacency(&mut self, el_id: u64, direction: GridDir, adj_id: u64) {
        match self.inner.entry(el_id) {
            std::collections::hash_map::Entry::Occupied(mut e) => {
                e.get_mut().add_at_dir(direction, adj_id)
            }
            std::collections::hash_map::Entry::Vacant(e) => {
                let mut adjacencies = Adjacencies::new();
                adjacencies.add_at_dir(direction, adj_id);
                e.insert(adjacencies);
            }
        }
    }

    pub(crate) fn get_all_adjacencies_in_direction(
        &self,
        el_id: &u64,
        direction: &GridDir,
    ) -> impl Iterator<Item = &u64> {
        self.inner
            .get(el_id)
            .expect("cannot get adjacencies for provided `el_id`")[*direction]
            .iter()
    }
}

/// Basic Subscriber for debugging purposes.
///
/// Implements [`overlap::Subscriber`] and [`singular::Subscriber`], making it usable with both resolvers.
/// Upon collapsing a tile, it will print the collapsed `GridPosition`, `tile_type_id` and (if applicable) `pattern_id`.
#[derive(Clone, Debug, Default)]
pub struct DebugSubscriber;

impl singular::Subscriber for DebugSubscriber {
    fn on_collapse(&mut self, position: &GridPosition, tile_type_id: u64) {
        println!("collapsed tile_type_id: {tile_type_id} on position: {position:?}");
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl overlap::Subscriber for DebugSubscriber {
    fn on_collapse(&mut self, position: &GridPosition, tile_type_id: u64, pattern_id: u64) {
        println!(
            "collapsed tile_type_id: {tile_type_id}, pattern_id: {pattern_id} on position: {position:?}"
        );
    }
}

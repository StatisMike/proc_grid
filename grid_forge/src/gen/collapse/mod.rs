mod error;
// pub mod overlap;
mod grid;
mod option;
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

use crate::map::GridDir;

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

    #[inline(always)]
    pub fn is_at_dir(&self, direction: &GridDir, id: &u64) -> bool {
        self.inner.get(*direction as usize).unwrap().contains(id)
    }

    #[inline(always)]
    pub fn any_at_dir(&self, direction: &GridDir, ids: &[u64]) -> bool {
        let bind = self.inner.get(*direction as usize).unwrap();
        ids.iter().any(|id| bind.contains(id))
    }
}

impl Index<GridDir> for Adjacencies {
    type Output = IntSet<u64>;

    fn index(&self, index: GridDir) -> &Self::Output {
        &self.inner[index as usize]
    }
}

#[derive(Clone, Debug, Default)]
pub struct AdjacencyTable {
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

    pub(crate) fn all_ids(&self) -> Vec<u64> {
        self.inner.keys().copied().collect()
    }

    pub(crate) fn check_adjacency(&self, el_id: &u64, direction: &GridDir, other_id: &u64) -> bool {
        if let Some(adjacencies) = self.inner.get(el_id) {
            return adjacencies.is_at_dir(direction, other_id);
        }
        false
    }

    pub(crate) fn check_adjacency_any(
        &self,
        el_id: &u64,
        direction: &GridDir,
        other_ids: &[u64],
    ) -> bool {
        if let Some(adjacencies) = self.inner.get(el_id) {
            return adjacencies.any_at_dir(direction, other_ids);
        }
        false
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

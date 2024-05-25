mod error;
mod frequency;
mod overlapping;
mod queue;
mod resolver;
mod rules;
mod subscriber;
mod tile;

use std::collections::{HashMap, HashSet};

// Flattened reexports
pub use error::CollapseError;
pub use frequency::FrequencyHints;
pub use overlapping::{analyzer::*, pattern::*, rules::*};
pub use queue::*;
pub use resolver::CollapsibleResolver;
pub use rules::*;
pub use tile::CollapsibleTileData;

use crate::map::GridDir;

#[derive(Clone, Debug, Default)]
pub(crate) struct Adjacencies {
    inner: Vec<HashSet<u64>>,
}

impl Adjacencies {
    pub fn new() -> Self {
        let mut inner = Vec::new();

        for _ in 0..GridDir::ALL_2D.len() {
            inner.push(HashSet::new());
        }

        Self { inner }
    }

    pub fn add_at_dir(&mut self, direction: GridDir, id: u64) {
        let v = self.inner.get_mut(direction as usize).unwrap();
        v.insert(id);
    }

    pub fn is_at_dir(&self, direction: GridDir, id: u64) -> bool {
        self.inner.get(direction as usize).unwrap().contains(&id)
    }

    pub fn any_at_dir(&self, direction: GridDir, ids: &[u64]) -> bool {
        let bind = self.inner.get(direction as usize).unwrap();
        ids.iter().any(|id| bind.contains(id))
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct AdjacencyTable {
    inner: HashMap<u64, Adjacencies>,
}

impl AdjacencyTable {
    pub fn insert_adjacency(&mut self, el_id: u64, direction: GridDir, adj_id: u64) {
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

    pub fn check_adjacency(&self, el_id: u64, direction: GridDir, other_id: u64) -> bool {
        if let Some(adjacencies) = self.inner.get(&el_id) {
            return adjacencies.is_at_dir(direction, other_id);
        }
        false
    }

    pub fn check_adjacency_any(&self, el_id: u64, direction: GridDir, other_ids: &[u64]) -> bool {
        if let Some(adjacencies) = self.inner.get(&el_id) {
            return adjacencies.any_at_dir(direction, other_ids);
        }
        false
    }
}

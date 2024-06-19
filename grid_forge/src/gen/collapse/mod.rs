//! # Collapsible procedural generation
//! 
//! Algorithms contained within this module works by having a collection of tiles, which final identity is unknown in the beginning -
//! *every position can be any tile*. In the process of generating a new grid, algorithm will *collapse* possible options for position,
//! choosing randomly one of them. Collapsed tile will then put contraints over the remaining tiles, reducing the number of possible options.
//! Process will then continue until all tiles are collapsed into one possible option.
//! 
//! This process is called elsewhere most often *Model Synthesis* or *Wave Function Collapse*, though the distinction between the two is
//! not always clear. Implementation contained there can be tailored to specific needs for the generation process, which makes the line
//! even more blurry, so new name is used here.
//! 
//! ## Main distinction
//! 
//! Two main types of algorithms are provided in distinct submodules:
//! - [`singular`] - implementation that can be used to generate maps. Each tile type needs to be self-descriptive in 
//! regards to its possible neighbours. Constraints for this type of generation is based strictly on possible neighbours for each given tile,
//! set of rules which can be described as: tile `X` can be placed in direction `D` of tile `Y`.
//! - [`overlap`] - implementation that can be used to generate maps based on observed patterns. Direct possible adjacents of each tile are
//! less important there, the pattern context is more important. First step for this type of generation is to create a collection of patterns
//! from sample gridmaps, which then will be tested for possible neigbours. During the process the possible patterns will then be collapsed
//! instead of individual tiles.
//! 
//! ## Struct types
//! 
//! Structs contained in submodules mentioned above are categorized in analogous way, with some additional distinctions described further
//! in documentation for concrete struct. 
//! 
//! - *adjacency rules* are the rules that are used to determine possible neighbours for each tile.
//! - *frequency hints* can be described as a weights per option - they can be derived from sample gridmaps and are a way to influence the
//! frequency of options occuring in the generated grid.
//! - *analyzers* provide methods for analyzing sample gridmaps and creating rulesets out of them.
//! - *collapsible grids* are the source of information for the *resolvers*, from the collection of collapsible tiles to the prepared 
//! adjacency rules and frequency hints. They can also contain some pre-collapsed tiles, providing initial constraints for the generation.
//! - *resolvers* are the main executors of the algorithm and are responsible for collapsing the tiles in the *collapsible grids*.
//! - *queues* are used to determine the order in which tiles are collapsed: [`PositionQueue`] takes next position to collapse in a fixed
//! order, while [`EntrophyQueue`] fetch the next position to collapse with the lowest entrophy.

mod error;
mod grid;
mod option;
pub mod overlap;
mod queue;
pub mod singular;
mod tile;

use std::{collections::HashSet, ops::Index};

// Flattened reexports
pub use error::CollapseError;
pub use grid::{CollapsedGrid, CollapsibleGrid};
pub use queue::*;
pub use tile::*;

use crate::{map::GridDir, tile::GridPosition};

#[derive(Clone, Debug, Default)]
pub(crate) struct Adjacencies {
    inner: Vec<HashSet<u64>>,
}

impl Adjacencies {
    pub fn new() -> Self {
        let mut inner = Vec::new();

        for _ in 0..GridDir::ALL_2D.len() {
            inner.push(HashSet::default());
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
    type Output = HashSet<u64>;

    fn index(&self, index: GridDir) -> &Self::Output {
        &self.inner[index as usize]
    }
}

/// Basic Subscriber for debugging purposes.
///
/// Implements both [`overlap::Subscriber`] and [`singular::Subscriber`], making it usable with both resolvers.
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

pub(crate) mod private {
    use std::collections::HashMap;

    use crate::map::GridDir;

    use super::Adjacencies;

    #[derive(Clone, Debug, Default)]
    pub struct AdjacencyTable {
        inner: HashMap<u64, Adjacencies>,
    }

    impl AsRef<HashMap<u64, Adjacencies>> for AdjacencyTable {
        fn as_ref(&self) -> &HashMap<u64, Adjacencies> {
            &self.inner
        }
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
}

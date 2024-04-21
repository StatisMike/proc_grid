use crate::{
    map::{GridDir, GridMap2D},
    tile::GridTile2D,
};

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    hash::{DefaultHasher, Hash, Hasher},
};

pub trait WFCTile
where
    Self: GridTile2D,
{
    fn wfc_id(&self) -> u64;
}

impl<T> WFCTile for T
where
    T: GridTile2D + Hash + Clone,
{
    fn wfc_id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        let mut cloned = self.clone();
        cloned.set_grid_position((0, 0));
        cloned.hash(&mut hasher);
        hasher.finish()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct WFCNeighbours {
    inner: HashMap<GridDir, BTreeSet<u64>>,
}

impl Default for WFCNeighbours {
    fn default() -> Self {
        let mut inner = HashMap::new();
        for dir in GridDir::ALL {
            inner.insert(*dir, BTreeSet::new());
        }
        Self { inner }
    }
}

impl WFCNeighbours {
    pub fn insert_at_dir(&mut self, dir: &GridDir, hash: u64) {
        let neighbours = self.inner.get_mut(dir).unwrap();

        if !neighbours.contains(&hash) {
            neighbours.insert(hash);
        }
    }

    pub fn is_valid_at_dir(&self, dir: &GridDir, option: u64) -> bool {
        let options_at_dir = self.inner
            .get(dir)
            .expect("no valid tiles at direction");
        options_at_dir.contains(&option)
    }
}

struct WFCModule<T>
where
    T: GridTile2D + PartialEq + Hash + Clone,
{
    hash: u64,
    grid: GridMap2D<T>,
    neighbours: WFCNeighbours,
}

impl<T> WFCModule<T>
where
    T: GridTile2D + PartialEq + Hash + Clone,
{
    fn new(grid: GridMap2D<T>) -> Self {
        let mut hasher = DefaultHasher::new();

        for position in grid.size().get_all_possible_positions() {
            if let Some(tile) = grid.get_tile_at_position(&position) {
                let mut cloned = tile.clone();
                cloned.set_grid_position((0, 0));
                Some(cloned).hash(&mut hasher)
            } else {
                None::<T>.hash(&mut hasher)
            };
        }

        let hash = hasher.finish();

        Self {
            hash,
            grid,
            neighbours: WFCNeighbours::default(),
        }
    }

    pub fn hash(&self) -> u64 {
        self.hash
    }

    pub fn grid(&self) -> &GridMap2D<T> {
        &self.grid
    }

    pub fn mut_neighbours(&mut self) -> &mut WFCNeighbours {
        &mut self.neighbours
    }

    pub fn neighbours(&self) -> &WFCNeighbours {
        &self.neighbours
    }
}

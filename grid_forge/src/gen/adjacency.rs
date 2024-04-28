use std::{
    collections::{BTreeSet, HashMap},
    marker::PhantomData,
};

use crate::{
    map::{GridDir, GridMap2D},
    tile::{GridTile2D, identifiable::IdentifiableTile},
};

use super::frequency::FrequencyRules;

pub trait AdjacencyAnalyzer<T>
where
    T: IdentifiableTile,
{
    fn adjacency(&self) -> &AdjacencyRules<T>;
    fn frequency(&self) -> &FrequencyRules<T>;
    fn tiles(&self) -> Vec<u64>;
    fn analyze(&mut self, map: &GridMap2D<T>);
}

#[derive(Debug)]
pub struct AdjacencyRules<T>
where
    T: IdentifiableTile,
{
    inner: HashMap<u64, InnerAdjacency>,
    id_type: PhantomData<T>,
}

impl<T> Clone for AdjacencyRules<T>
where
    T: IdentifiableTile,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            id_type: PhantomData::<T>,
        }
    }
}

impl<T> Default for AdjacencyRules<T>
where
    T: IdentifiableTile,
{
    fn default() -> Self {
        Self {
            inner: HashMap::new(),
            id_type: PhantomData::<T>,
        }
    }
}

impl<T> AdjacencyRules<T>
where
    T: IdentifiableTile,
{
    pub fn add_adjacency(&mut self, tile: &T, adjacent_tile: &T, direction: GridDir) {
        self.add_adjacency_raw(tile.get_tile_id(), adjacent_tile.get_tile_id(), direction)
    }

    pub fn debug_print(&self) {
        for (tile_id, rules) in self.inner.iter() {
            println!("Id: {tile_id}; Rules: {rules:?}")
        }
    }

    pub(crate) fn is_adjacent_option_valid(
        &self,
        tile_id: u64,
        adjacent_id: u64,
        direction: GridDir,
    ) -> bool {
        self.is_valid_raw(adjacent_id, tile_id, direction.opposite())
    }

    pub(crate) fn is_adjacent_option_valid_for_options(
        &self,
        tile_options: &[u64],
        adjacent_id: u64,
        direction: GridDir,
    ) -> bool {
        tile_options
            .iter()
            .any(|id| self.is_adjacent_option_valid(adjacent_id, *id, direction))
    }

    pub(crate) fn add_adjacency_raw(&mut self, tile_id: u64, adjacent_id: u64, direction: GridDir) {
        let adjacents = self.inner.entry(tile_id).or_default();

        adjacents.add_option(adjacent_id, direction);
    }

    pub(crate) fn is_valid_raw(&self, tile_id: u64, adjacent_id: u64, direction: GridDir) -> bool {
        if let Some(rules) = self.inner.get(&tile_id) {
            rules.is_in_options(adjacent_id, direction)
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
struct InnerAdjacency {
    ia: HashMap<GridDir, BTreeSet<u64>>,
}

impl Default for InnerAdjacency {
    fn default() -> Self {
        let mut ia = HashMap::new();
        for dir in GridDir::ALL {
            ia.insert(*dir, BTreeSet::new());
        }
        Self { ia }
    }
}

impl InnerAdjacency {
    fn add_option(&mut self, adjacent_id: u64, dir: GridDir) {
        let opts = self.ia.get_mut(&dir).unwrap();
        opts.insert(adjacent_id);
    }

    fn is_in_options(&self, adjacent_id: u64, dir: GridDir) -> bool {
        self.ia.get(&dir).unwrap().contains(&adjacent_id)
    }
}

#[cfg(test)]
mod test {
    use crate::{map::GridDir, tile::GridTile2D, GridPos2D};

    use super::{AdjacencyRules, IdentifiableTile};

    #[derive(Clone, Copy)]
    struct TestTile {
        pos: GridPos2D,
        id: u64,
    }

    impl GridTile2D for TestTile {
        fn grid_position(&self) -> GridPos2D {
            self.pos
        }

        fn set_grid_position(&mut self, position: GridPos2D) {
            self.pos = position;
        }
    }

    impl IdentifiableTile for TestTile {
        fn get_tile_id(&self) -> u64 {
            self.id
        }
    }

    impl TestTile {
        fn new(id: u64) -> Self {
            Self { pos: (0, 0), id }
        }
    }

    #[test]
    fn can_add_and_evaluate_rules() {
        let main_tile = TestTile::new(0);
        let other_tile = TestTile::new(1);

        let mut adjacency_rules = AdjacencyRules::default();

        for dir in GridDir::ALL {
            adjacency_rules.add_adjacency(&main_tile, &other_tile, *dir);

            assert!(adjacency_rules.is_valid_raw(
                main_tile.get_tile_id(),
                other_tile.get_tile_id(),
                *dir
            ));
            assert!(!adjacency_rules.is_valid_raw(
                other_tile.get_tile_id(),
                main_tile.get_tile_id(),
                *dir
            ));

            assert!(adjacency_rules.is_adjacent_option_valid(
                other_tile.get_tile_id(),
                main_tile.get_tile_id(),
                dir.opposite()
            ));
        }
    }
}

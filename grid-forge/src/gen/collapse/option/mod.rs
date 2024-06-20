use std::{
    collections::{BTreeMap, HashMap},
    ops::{Index, IndexMut},
};

use crate::{
    map::{DirectionTable, GridDir},
    tile::identifiable::collection::IdentTileCollection,
};

use super::private::AdjacencyTable;

#[derive(Debug, Clone)]
pub struct PerOptionTable<T> {
    table: Vec<T>,
}

impl<T> Index<usize> for PerOptionTable<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.table.index(index)
    }
}

impl<T> IndexMut<usize> for PerOptionTable<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.table.index_mut(index)
    }
}

impl<T> Default for PerOptionTable<T> {
    fn default() -> Self {
        Self {
            table: Default::default(),
        }
    }
}

impl<T> AsRef<Vec<T>> for PerOptionTable<T> {
    fn as_ref(&self) -> &Vec<T> {
        &self.table
    }
}

impl<T> AsMut<Vec<T>> for PerOptionTable<T> {
    fn as_mut(&mut self) -> &mut Vec<T> {
        &mut self.table
    }
}

#[derive(Debug, Default, Clone)]
pub struct PerOptionData {
    option_map: HashMap<u64, usize>,
    option_map_rev: HashMap<u64, u64>,
    adjacencies: PerOptionTable<DirectionTable<Vec<usize>>>,
    ways_to_be_option: WaysToBeOption,
    opt_with_weight: PerOptionTable<(u32, f32)>,
    option_count: usize,
    possible_options_count: usize,
}

impl IdentTileCollection for PerOptionData {
    type DATA = usize;

    fn inner(&self) -> &HashMap<u64, Self::DATA> {
        &self.option_map
    }

    fn inner_mut(&mut self) -> &mut HashMap<u64, Self::DATA> {
        &mut self.option_map
    }

    fn rev(&self) -> &HashMap<u64, u64> {
        &self.option_map_rev
    }

    fn rev_mut(&mut self) -> &mut HashMap<u64, u64> {
        &mut self.option_map_rev
    }
}

impl PerOptionData {
    pub fn populate(
        &mut self,
        options_with_weights: &BTreeMap<u64, u32>,
        adjacencies: &AdjacencyTable,
    ) {
        for (n, (option_id, option_weight)) in options_with_weights.iter().enumerate() {
            self.add_tile_data(*option_id, n);

            self.opt_with_weight.as_mut().push((
                *option_weight,
                (*option_weight as f32) * (*option_weight as f32).log2(),
            ));
        }

        self.option_count = self.option_map.len();
        self.possible_options_count = self.option_count;

        for trans_id in 0..self.option_count {
            let original_id = self.get_tile_type_id(&trans_id).unwrap();
            self.adjacencies
                .table
                .push(self.translate_adjacency_table(original_id, adjacencies));
        }

        self.generate_ways_to_be_option();
    }

    pub fn get_all_enabled_in_direction(&self, option_id: usize, direction: GridDir) -> &[usize] {
        &self.adjacencies[option_id][direction]
    }

    pub fn iter_weights(&self) -> impl Iterator<Item = (usize, &(u32, f32))> {
        self.opt_with_weight.table.iter().enumerate()
    }

    pub fn get_weights(&self, option_idx: usize) -> (u32, f32) {
        self.opt_with_weight.table[option_idx]
    }

    pub fn num_options(&self) -> usize {
        self.option_count
    }

    pub fn num_possible_options(&self) -> usize {
        self.possible_options_count
    }

    pub fn get_ways_to_become_option(&self) -> &WaysToBeOption {
        &self.ways_to_be_option
    }

    fn generate_ways_to_be_option(&mut self) {
        let inner = self.ways_to_be_option.mut_inner().as_mut();
        for adj in self.adjacencies.table.iter() {
            let table = DirectionTable::new_array([
                adj.index(GridDir::UP).len(),
                adj.index(GridDir::DOWN).len(),
                adj.index(GridDir::LEFT).len(),
                adj.index(GridDir::RIGHT).len(),
            ]);
            if table.inner().contains(&0) {
                self.possible_options_count -= 1;
                inner.push(WaysToBeOption::EMPTY_DIR_TABLE)
            } else {
                inner.push(table);
            }
        }
    }

    fn translate_adjacency_table(
        &self,
        original_id: u64,
        adjacencies: &AdjacencyTable,
    ) -> DirectionTable<Vec<usize>> {
        let mut table = DirectionTable::default();
        for direction in GridDir::ALL_2D {
            table[*direction] = Vec::from_iter(
                adjacencies
                    .get_all_adjacencies_in_direction(&original_id, direction)
                    .map(|original_id: &u64| {
                        self.get_tile_data(original_id)
                            .expect("cannot get mapped id")
                    })
                    .copied(),
            );
        }
        table
    }
}

#[derive(Clone, Debug, Default)]
pub struct WaysToBeOption {
    table: PerOptionTable<DirectionTable<usize>>,
}

impl WaysToBeOption {
    pub(crate) const EMPTY_DIR_TABLE: DirectionTable<usize> =
        DirectionTable::new_array(Self::EMPTY_TABLE);

    pub(crate) const EMPTY_TABLE: [usize; 4] = [0, 0, 0, 0];

    /// Decrements number of ways to become option from given direction. If reaches
    /// 0, returns `true` and given option should be removed.
    pub(crate) fn decrement(&mut self, option_idx: usize, direction: GridDir) -> bool {
        // let num_ways_by_dir = self.table.index_mut(option_idx);
        // let num_ways = num_ways_by_dir[direction];
        if self.table[option_idx][direction] == 0 {
            return false;
        }
        self.table[option_idx][direction] -= 1;
        if self.table[option_idx][direction] > 0 {
            return false;
        }
        self.table[option_idx] = Self::EMPTY_DIR_TABLE;
        true
    }

    pub(crate) fn mut_inner(&mut self) -> &mut PerOptionTable<DirectionTable<usize>> {
        &mut self.table
    }

    pub(crate) fn iter_possible(&self) -> impl Iterator<Item = usize> + '_ {
        self.table
            .as_ref()
            .iter()
            .enumerate()
            .filter_map(|(idx, t)| if t[GridDir::UP] == 0 { None } else { Some(idx) })
    }

    pub(crate) fn purge_others(&mut self, options: &[usize]) {
        for (option_id, ways) in self.table.as_mut().iter_mut().enumerate() {
            if options.contains(&option_id) {
                continue;
            }
            *ways = Self::EMPTY_DIR_TABLE;
        }
    }

    pub(crate) fn purge_option(&mut self, option_idx: usize) -> bool {
        if self.table[option_idx].inner().iter().all(|i| i == &0) {
            return false;
        }
        self.table[option_idx] = Self::EMPTY_DIR_TABLE;
        true
    }
}

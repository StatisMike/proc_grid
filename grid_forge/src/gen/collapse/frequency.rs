use std::collections::BTreeMap;
use std::marker::PhantomData;

use crate::map::GridMap2D;
use crate::tile::identifiable::IdentifiableTile;

#[derive(Debug)]
pub struct FrequencyHints<T>
where
    T: IdentifiableTile,
{
    weights: BTreeMap<u64, u32>,
    id_type: PhantomData<T>,
}

impl<T> Clone for FrequencyHints<T>
where
    T: IdentifiableTile,
{
    fn clone(&self) -> Self {
        Self {
            weights: self.weights.clone(),
            id_type: PhantomData::<T>,
        }
    }
}

impl<T> Default for FrequencyHints<T>
where
    T: IdentifiableTile,
{
    fn default() -> Self {
        Self {
            weights: BTreeMap::new(),
            id_type: PhantomData::<T>,
        }
    }
}

impl<T> FrequencyHints<T>
where
    T: IdentifiableTile,
{
    pub fn set_weight_for_tile(&mut self, tile: &T, weight: u32) {
        let entry = self.weights.entry(tile.tile_type_id()).or_default();
        *entry = weight;
    }

    pub fn count_tile(&mut self, tile: &T) {
        if let Some(count) = self.weights.get_mut(&tile.tile_type_id()) {
            *count += 1;
        } else {
            self.weights.insert(tile.tile_type_id(), 1);
        }
    }

    pub(crate) fn get_all_weights_cloned(&self) -> BTreeMap<u64, u32> {
        self.weights.clone()
    }

    pub fn analyze_grid_map(&mut self, map: &GridMap2D<T>) {
        for position in map.get_all_positions() {
            self.count_tile(map.get_tile_at_position(&position).unwrap())
        }
    }
}

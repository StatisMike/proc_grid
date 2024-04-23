use std::{
    collections::{BTreeMap, HashMap},
    marker::PhantomData,
};

use super::adjacency::IdentifiableTile;

#[derive(Debug)]
pub struct FrequencyRules<T>
where
    T: IdentifiableTile,
{
    weights: BTreeMap<u64, u32>,
    id_type: PhantomData<T>,
}

impl<T> Clone for FrequencyRules<T>
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

impl<T> Default for FrequencyRules<T>
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

impl<T> FrequencyRules<T>
where
    T: IdentifiableTile,
{
    pub fn debug_print(&self) {
        println!("{:?}", self.weights)
    }

    pub fn count_tile(&mut self, tile: &T) {
        if let Some(count) = self.weights.get_mut(&tile.get_tile_id()) {
            *count += 1;
        } else {
            self.weights.insert(tile.get_tile_id(), 1);
        }
    }

    pub(crate) fn get_all_weights_cloned(&self) -> BTreeMap<u64, u32> {
        self.weights.clone()
    }

    pub(crate) fn get_weight_sum(&self, ids: &[u64]) -> u32 {
        let mut sum = 0;
        for id in ids {
            if let Some(weight) = self.weights.get(id) {
                sum += weight;
            }
        }
        sum
    }
}

use std::{
    collections::{BTreeMap, HashMap},
    marker::PhantomData,
};

use crate::tile::identifiable::IdentifiableTile;

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
}

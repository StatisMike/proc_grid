use std::collections::BTreeMap;
use std::marker::PhantomData;

use crate::map::GridMap2D;
use crate::tile::identifiable::IdentifiableTileData;
use crate::tile::TileContainer;

#[derive(Debug)]
pub struct FrequencyHints<Data>
where
    Data: IdentifiableTileData,
{
    weights: BTreeMap<u64, u32>,
    id_type: PhantomData<Data>,
}

impl<Data> Clone for FrequencyHints<Data>
where
    Data: IdentifiableTileData,
{
    fn clone(&self) -> Self {
        Self {
            weights: self.weights.clone(),
            id_type: PhantomData::<Data>,
        }
    }
}

impl<T> Default for FrequencyHints<T>
where
    T: IdentifiableTileData,
{
    fn default() -> Self {
        Self {
            weights: BTreeMap::new(),
            id_type: PhantomData::<T>,
        }
    }
}

impl<Data> FrequencyHints<Data>
where
    Data: IdentifiableTileData,
{
    pub fn set_weight_for_tile<Tile>(&mut self, tile: &Tile, weight: u32)
    where
        Tile: TileContainer + AsRef<Data>,
    {
        let entry = self
            .weights
            .entry(tile.as_ref().tile_type_id())
            .or_default();
        *entry = weight;
    }

    pub fn count_tile<Tile>(&mut self, tile: &Tile)
    where
        Tile: TileContainer + AsRef<Data>,
    {
        if let Some(count) = self.weights.get_mut(&tile.as_ref().tile_type_id()) {
            *count += 1;
        } else {
            self.weights.insert(tile.as_ref().tile_type_id(), 1);
        }
    }

    pub(crate) fn get_all_weights_cloned(&self) -> BTreeMap<u64, u32> {
        self.weights.clone()
    }

    pub fn analyze_grid_map(&mut self, map: &GridMap2D<Data>) {
        for position in map.get_all_positions() {
            let reference = map.get_tile_at_position(&position).unwrap();
            self.count_tile(&reference)
        }
    }
}

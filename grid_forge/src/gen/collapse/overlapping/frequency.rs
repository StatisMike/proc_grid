use std::{collections::BTreeMap, marker::PhantomData};

use crate::{
    gen::collapse::AdjacencyTable,
    map::GridDir,
    tile::identifiable::{collection::IdentTileCollection, IdentifiableTileData},
};

use super::pattern::{OverlappingPatternGrid, Pattern, PatternCollection, PatternTileData};

#[derive(Clone, Debug)]
pub struct PatternFrequencyHints<const P_X: usize, const P_Y: usize, const P_Z: usize, Data>
where
    Data: IdentifiableTileData,
{
    weights: BTreeMap<u64, u32>,
    id_type: PhantomData<fn(Data)>,
}

impl<const P_X: usize, const P_Y: usize, const P_Z: usize, Data>
Default for PatternFrequencyHints<P_X, P_Y, P_Z, Data>
where
    Data: IdentifiableTileData,
    {
      fn default() -> Self {
        Self {
          weights: BTreeMap::new(),
          id_type: PhantomData
        }    
      }
      
    }

impl<Data, const P_X: usize, const P_Y: usize, const P_Z: usize>
    PatternFrequencyHints<P_X, P_Y, P_Z, Data>
where
    Data: IdentifiableTileData,
{
    pub fn set_weight_for_pattern(&mut self, pattern: &Pattern<P_X, P_Y, P_Z>, weight: u32) {
        let entry = self.weights.entry(pattern.pattern_id).or_default();
        *entry = weight;
    }

    pub(crate) fn count_pattern(&mut self, pattern_id: u64) {
        if let Some(count) = self.weights.get_mut(&pattern_id) {
            *count += 1;
        } else {
            self.weights.insert(pattern_id, 1);
        }
    }

    pub(crate) fn get_all_weights_cloned(&self) -> BTreeMap<u64, u32> {
        self.weights.clone()
    }

    pub fn get_weight_for_pattern(&self, pattern_id: u64) -> u32 {
        *self.weights.get(&pattern_id).unwrap_or(&0)
    }

    pub fn analyze_pattern_grid(&mut self, grid: &OverlappingPatternGrid<P_X, P_Y, P_Z>) {
        for tile in grid.inner().iter_tiles() {
            if let PatternTileData::WithPattern {
                tile_type_id: _,
                pattern_id,
            } = tile.as_ref()
            {
                self.count_pattern(*pattern_id);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct PatternAdjacencyRules<const P_X: usize, const P_Y: usize, const P_Z: usize, Data>
where
    Data: IdentifiableTileData,
{
    inner: AdjacencyTable,
    data_type: PhantomData<fn(Data)>,
}

impl<const P_X: usize, const P_Y: usize, const P_Z: usize, Data>
    Default for PatternAdjacencyRules<P_X, P_Y, P_Z, Data>
where
    Data: IdentifiableTileData,
{
  fn default() -> Self {
      Self {
        inner: AdjacencyTable::default(),
        data_type: PhantomData
      }
  }
}

impl<const P_X: usize, const P_Y: usize, const P_Z: usize, Data>
    PatternAdjacencyRules<P_X, P_Y, P_Z, Data>
where
    Data: IdentifiableTileData,
{
    pub fn analyze_collection(&mut self, collection: &PatternCollection<P_X, P_Y, P_Z>) {
        for (id_outer, pat_outer) in collection.inner().iter() {
            for (id_inner, pat_inner) in collection.inner().iter() {
                for direction in GridDir::ALL_2D {
                    if pat_outer.is_compatible_with(pat_inner, *direction) {
                        self.inner
                            .insert_adjacency(*id_outer, *direction, *id_inner);
                    }
                }
            }
        }
    }
}

impl<const P_X: usize, const P_Y: usize, const P_Z: usize, Data> AsRef<AdjacencyTable>
    for PatternAdjacencyRules<P_X, P_Y, P_Z, Data>
where
    Data: IdentifiableTileData,
{
    fn as_ref(&self) -> &AdjacencyTable {
        &self.inner
    }
}

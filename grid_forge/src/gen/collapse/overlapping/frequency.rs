use std::{collections::BTreeMap, marker::PhantomData};

use crate::{
    gen::collapse::AdjacencyTable,
    map::GridDir,
    tile::identifiable::{collection::IdentTileCollection, IdentifiableTileData},
};

use super::pattern::{OverlappingPatternGrid, Pattern, PatternCollection, PatternTileData};

#[derive(Clone, Debug, Default)]
pub struct PatternFrequencyHints<
    const PATTERN_WIDTH: usize,
    const PATTERN_HEIGHT: usize,
    const PATTERN_DEPTH: usize,
    Data,
> where
    Data: IdentifiableTileData,
{
    weights: BTreeMap<u64, u32>,
    id_type: PhantomData<*const Data>,
}

impl<Data, const PATTERN_WIDTH: usize, const PATTERN_HEIGHT: usize, const PATTERN_DEPTH: usize>
    PatternFrequencyHints<PATTERN_WIDTH, PATTERN_HEIGHT, PATTERN_DEPTH, Data>
where
    Data: IdentifiableTileData,
{
    pub fn set_weight_for_pattern(
        &mut self,
        pattern: &Pattern<PATTERN_WIDTH, PATTERN_HEIGHT, PATTERN_DEPTH>,
        weight: u32,
    ) {
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

    pub fn analyze_pattern_grid(
        &mut self,
        grid: &OverlappingPatternGrid<PATTERN_WIDTH, PATTERN_HEIGHT, PATTERN_DEPTH>,
    ) {
        for tile in grid.inner().iter_tiles() {
            if let PatternTileData::WithPattern {
                tile_type_id: _,
                pattern_id: pattern_id,
            } = tile.as_ref()
            {
                self.count_pattern(*pattern_id);
            }
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct PatternAdjacencyRules<
    const PATTERN_WIDTH: usize,
    const PATTERN_HEIGHT: usize,
    const PATTERN_DEPTH: usize,
    Data,
> where
    Data: IdentifiableTileData,
{
    inner: AdjacencyTable,
    data_type: PhantomData<*const Data>,
}

impl<const PATTERN_WIDTH: usize, const PATTERN_HEIGHT: usize, const PATTERN_DEPTH: usize, Data>
    PatternAdjacencyRules<PATTERN_WIDTH, PATTERN_HEIGHT, PATTERN_DEPTH, Data>
where
    Data: IdentifiableTileData,
{
    pub fn analyze_collection(
        &mut self,
        collection: &PatternCollection<PATTERN_WIDTH, PATTERN_HEIGHT, PATTERN_DEPTH>,
    ) {
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

impl<const PATTERN_WIDTH: usize, const PATTERN_HEIGHT: usize, const PATTERN_DEPTH: usize, Data>
    AsRef<AdjacencyTable>
    for PatternAdjacencyRules<PATTERN_WIDTH, PATTERN_HEIGHT, PATTERN_DEPTH, Data>
where
    Data: IdentifiableTileData,
{
    fn as_ref(&self) -> &AdjacencyTable {
        &self.inner
    }
}

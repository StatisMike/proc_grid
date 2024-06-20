use crate::map::{GridDir, GridMap2D};
use crate::tile::identifiable::collection::IdentTileCollection;
use crate::tile::identifiable::IdentifiableTileData;

use super::pattern::{
    OverlappingPattern, OverlappingPatternGrid, PatternCollection, PatternTileData,
};

use std::collections::BTreeMap;
use std::marker::PhantomData;

use crate::gen::collapse::private::AdjacencyTable;

/// GridMap analyzer for overlapping pattern data.
///
/// It allows analyzing the [`GridMap2D`] of [`IdentifiableTileData`], producing all elements necessary for
/// creation of [`CollapsiblePatternGrid`](crate::gen::collapse::overlap::CollapsiblePatternGrid) for
/// [`overlap::Resolver`](crate::gen::collapse::overlap::Resolver) to collapse into new map.
pub struct Analyzer<P, Data>
where
    P: OverlappingPattern,
    Data: IdentifiableTileData,
{
    collection: PatternCollection<P>,
    frequency: FrequencyHints<P, Data>,
    adjacency: AdjacencyRules<P, Data>,
}

impl<P: OverlappingPattern, Data: IdentifiableTileData> Default for Analyzer<P, Data>
where
    Data: IdentifiableTileData,
{
    fn default() -> Self {
        Self {
            collection: Default::default(),
            frequency: Default::default(),
            adjacency: Default::default(),
        }
    }
}

impl<P: OverlappingPattern, Data: IdentifiableTileData> Analyzer<P, Data> {
    /// Analyzes the [`GridMap2D`] of [`IdentifiableTileData`], gathering elements necessary for creation of new
    /// [`CollapsiblePatternGrid`](crate::gen::collapse::overlap::CollapsiblePatternGrid) to collapse.
    ///
    /// Returns [`OverlappingPatternGrid`], which is a transformed source map if more insights about which patterns
    /// were discoveren in specific positions on the map.
    pub fn analyze(&mut self, map: &GridMap2D<Data>) -> OverlappingPatternGrid<P> {
        let grid = OverlappingPatternGrid::from_map(map, &mut self.collection);
        self.frequency.analyze_pattern_grid(&grid);
        self.adjacency.analyze_collection(&self.collection);

        grid
    }

    pub fn get_collection(&self) -> &PatternCollection<P> {
        &self.collection
    }

    pub fn get_frequency(&self) -> &FrequencyHints<P, Data> {
        &self.frequency
    }

    pub fn get_adjacency(&self) -> &AdjacencyRules<P, Data> {
        &self.adjacency
    }
}

/// Frequency hints for the *overlapping pattern*-based generative algorithm.
///
/// Describes the frequency of occurence of each discovered pattern. Generated automatically while analyzing sample
/// maps with [`Analyzer`], though afterwards frequencies could be tweaked manually.
#[derive(Debug)]
pub struct FrequencyHints<P, Data>
where
    P: OverlappingPattern,
    Data: IdentifiableTileData,
{
    weights: BTreeMap<u64, u32>,
    pattern_type: PhantomData<P>,
    data_type: PhantomData<Data>,
}

impl<P, Data> Clone for FrequencyHints<P, Data>
where
    P: OverlappingPattern,
    Data: IdentifiableTileData,
{
    fn clone(&self) -> Self {
        Self {
            weights: self.weights.clone(),
            pattern_type: self.pattern_type,
            data_type: self.data_type,
        }
    }
}

impl<P, Data> Default for FrequencyHints<P, Data>
where
    P: OverlappingPattern,
    Data: IdentifiableTileData,
{
    fn default() -> Self {
        Self {
            weights: BTreeMap::new(),
            pattern_type: PhantomData,
            data_type: PhantomData,
        }
    }
}

impl<P, Data> FrequencyHints<P, Data>
where
    P: OverlappingPattern,
    Data: IdentifiableTileData,
{
    pub fn set_weight_for_pattern(&mut self, pattern: &P, weight: u32) {
        let entry = self.weights.entry(pattern.pattern_id()).or_default();
        *entry = weight;
    }

    pub fn set_weight_for_pattern_id(&mut self, pattern_id: u64, weight: u32) {
        let entry = self.weights.entry(pattern_id).or_default();
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

    pub fn analyze_pattern_grid(&mut self, grid: &OverlappingPatternGrid<P>) {
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

/// Adjacency rules for the *overlapping pattern*-based generative algorithm.
///
/// Contrary to [`singular::AdjacencyRules`](crate::gen::collapse::singular::AdjacencyRules), these rules are not based directly on the
/// neighbouring tiles found within the sample maps, but on the `tile_type_id`s hold within the patterns.
///
/// Two patterns are considered compatible in given direction if the overlapping tiles contained within them are identical.
#[derive(Clone)]
pub struct AdjacencyRules<P, Data>
where
    P: OverlappingPattern,
    Data: IdentifiableTileData,
{
    inner: AdjacencyTable,
    pattern_type: PhantomData<P>,
    data_type: PhantomData<Data>,
}

impl<P, Data> core::fmt::Debug for AdjacencyRules<P, Data>
where
    P: OverlappingPattern,
    Data: IdentifiableTileData,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdjacencyRules")
            .field("inner", &self.inner)
            .field("pattern_type", &self.pattern_type)
            .field("data_type", &self.data_type)
            .finish()
    }
}

impl<P, Data> Default for AdjacencyRules<P, Data>
where
    P: OverlappingPattern,
    Data: IdentifiableTileData,
{
    fn default() -> Self {
        Self {
            inner: AdjacencyTable::default(),
            pattern_type: PhantomData,
            data_type: PhantomData,
        }
    }
}

impl<P, Data> AdjacencyRules<P, Data>
where
    P: OverlappingPattern,
    Data: IdentifiableTileData,
{
    /// Analyzes the [`PatternCollection`] to find out which patterns are compatible with each other.
    pub fn analyze_collection(&mut self, collection: &PatternCollection<P>) {
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

    pub(crate) fn inner(&self) -> &AdjacencyTable {
        &self.inner
    }

    pub fn is_valid_at_dir(
        &self,
        pattern_id: u64,
        direction: GridDir,
        other_pattern_id: u64,
    ) -> bool {
        let Some(adj) = self.inner().as_ref().get(&pattern_id) else {
            return false;
        };
        adj.inner[direction as usize].contains(&other_pattern_id)
    }
}

impl<P, Data> AsRef<AdjacencyTable> for AdjacencyRules<P, Data>
where
    P: OverlappingPattern,
    Data: IdentifiableTileData,
{
    fn as_ref(&self) -> &AdjacencyTable {
        &self.inner
    }
}

use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    marker::PhantomData,
};

use crate::{
    gen::collapse::{Adjacencies, AdjacencyTable, FrequencyHints},
    map::{GridDir, GridMap2D, GridSize},
    tile::{identifiable::IdentifiableTileData, GridPosition, TileContainer},
};

use super::{
    frequency::{PatternAdjacencyRules, PatternFrequencyHints},
    pattern::{OverlappingPatternGrid, Pattern, PatternCollection},
};

#[derive(Default)]
pub struct OverlappingAnalyzer<
    const PATTERN_WIDTH: usize,
    const PATTERN_HEIGHT: usize,
    const PATTERN_DEPTH: usize,
    Data: IdentifiableTileData,
> where
    Data: IdentifiableTileData,
{
    collection: PatternCollection<PATTERN_WIDTH, PATTERN_HEIGHT, PATTERN_DEPTH>,
    frequency: PatternFrequencyHints<PATTERN_WIDTH, PATTERN_HEIGHT, PATTERN_DEPTH, Data>,
    adjacency: PatternAdjacencyRules<PATTERN_WIDTH, PATTERN_HEIGHT, PATTERN_DEPTH, Data>,
}

impl<
        const PATTERN_WIDTH: usize,
        const PATTERN_HEIGHT: usize,
        const PATTERN_DEPTH: usize,
        Data: IdentifiableTileData,
    > OverlappingAnalyzer<PATTERN_WIDTH, PATTERN_HEIGHT, PATTERN_DEPTH, Data>
{
    pub fn analyze_map(
        &mut self,
        map: &GridMap2D<Data>,
    ) -> OverlappingPatternGrid<PATTERN_WIDTH, PATTERN_HEIGHT, PATTERN_DEPTH> {
        let grid = OverlappingPatternGrid::from_map(map, &mut self.collection);
        self.frequency.analyze_pattern_grid(&grid);
        self.adjacency.analyze_collection(&self.collection);

        grid
    }

    pub fn get_collection(
        &self,
    ) -> &PatternCollection<PATTERN_WIDTH, PATTERN_HEIGHT, PATTERN_DEPTH> {
        &self.collection
    }

    pub fn get_frequency(
        &self,
    ) -> &PatternFrequencyHints<PATTERN_WIDTH, PATTERN_HEIGHT, PATTERN_DEPTH, Data> {
        &self.frequency
    }

    pub fn get_adjacency(
        &self,
    ) -> &PatternAdjacencyRules<PATTERN_WIDTH, PATTERN_HEIGHT, PATTERN_DEPTH, Data> {
        &self.adjacency
    }
}

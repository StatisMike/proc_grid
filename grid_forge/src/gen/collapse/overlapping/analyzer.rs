use crate::{
    map::GridMap2D,
    tile::identifiable::IdentifiableTileData,
};

use super::{
    frequency::{PatternAdjacencyRules, PatternFrequencyHints},
    pattern::{OverlappingPatternGrid, PatternCollection},
};

pub struct OverlappingAnalyzer<
    const P_X: usize,
    const P_Y: usize,
    const P_Z: usize,
    Data: IdentifiableTileData,
> where
    Data: IdentifiableTileData,
{
    collection: PatternCollection<P_X, P_Y, P_Z>,
    frequency: PatternFrequencyHints<P_X, P_Y, P_Z, Data>,
    adjacency: PatternAdjacencyRules<P_X, P_Y, P_Z, Data>,
}

impl<const P_X: usize, const P_Y: usize, const P_Z: usize, Data: IdentifiableTileData>
    Default for OverlappingAnalyzer<P_X, P_Y, P_Z, Data>
where Data: IdentifiableTileData {
    fn default() -> Self {
        Self { collection: Default::default(), frequency: Default::default(), adjacency: Default::default() }
    }
}

impl<const P_X: usize, const P_Y: usize, const P_Z: usize, Data: IdentifiableTileData>
    OverlappingAnalyzer<P_X, P_Y, P_Z, Data>
{
    pub fn analyze_map(&mut self, map: &GridMap2D<Data>) -> OverlappingPatternGrid<P_X, P_Y, P_Z> {
        let grid = OverlappingPatternGrid::from_map(map, &mut self.collection);
        self.frequency.analyze_pattern_grid(&grid);
        self.adjacency.analyze_collection(&self.collection);

        grid
    }

    pub fn get_collection(&self) -> &PatternCollection<P_X, P_Y, P_Z> {
        &self.collection
    }

    pub fn get_frequency(&self) -> &PatternFrequencyHints<P_X, P_Y, P_Z, Data> {
        &self.frequency
    }

    pub fn get_adjacency(&self) -> &PatternAdjacencyRules<P_X, P_Y, P_Z, Data> {
        &self.adjacency
    }
}

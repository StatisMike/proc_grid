use std::collections::BTreeSet;

use crate::gen::adjacency::{AdjacencyAnalyzer, AdjacencyRules, IdentifiableTile};
use crate::gen::frequency::FrequencyRules;
use crate::map::{GridDir, GridMap2D};
use crate::GridPos2D;

pub struct WFCAnalyzer<T>
where
    T: IdentifiableTile,
{
    tiles: BTreeSet<u64>,
    frequency_rules: FrequencyRules<T>,
    adjacency_rules: AdjacencyRules<T>,
}

impl<T> Default for WFCAnalyzer<T>
where
    T: IdentifiableTile,
{
    fn default() -> Self {
        Self {
            tiles: BTreeSet::new(),
            frequency_rules: FrequencyRules::default(),
            adjacency_rules: AdjacencyRules::default(),
        }
    }
}

impl<T> WFCAnalyzer<T>
where
    T: IdentifiableTile,
{
    fn analyze_tile_at_pos(&mut self, map: &GridMap2D<T>, pos: GridPos2D) {
        if let Some(tile) = map.get_tile_at_position(&pos) {
            self.tiles.insert(tile.get_tile_id());
            self.frequency_rules.count_tile(tile);

            for dir in GridDir::ALL {
                if let Some(neighbour) = map.get_neighbour_at(&pos, dir) {
                    self.adjacency_rules.add_adjacency(tile, neighbour, *dir)
                }
            }
        }
    }

    pub fn adjacency(&self) -> &AdjacencyRules<T> {
        &self.adjacency_rules
    }

    pub fn frequency(&self) -> &FrequencyRules<T> {
        &self.frequency_rules
    }

    pub fn tiles(&self) -> Vec<u64> {
        Vec::from_iter(self.tiles.iter().cloned())
    }
}

impl<T> AdjacencyAnalyzer<T> for WFCAnalyzer<T>
where
    T: IdentifiableTile,
{
    fn analyze(&mut self, map: &GridMap2D<T>) {
        for position in map.get_all_positions() {
            self.analyze_tile_at_pos(map, position);
        }
    }

    fn adjacency(&self) -> &AdjacencyRules<T> {
        &self.adjacency_rules
    }

    fn frequency(&self) -> &FrequencyRules<T> {
        &self.frequency_rules
    }

    fn tiles(&self) -> Vec<u64> {
        Vec::from_iter(self.tiles.iter().cloned())
    }
}

#[cfg(test)]
pub(crate) mod test {
    use crate::{
        gen::{
            adjacency::{AdjacencyAnalyzer, IdentifiableTile},
            wfc::analyzer::WFCAnalyzer,
        },
        map::{GridMap2D, GridSize},
        tile::GridTile2D,
        GridPos2D,
    };

    #[derive(Debug, Clone)]
    pub(crate) struct TT {
        pos: GridPos2D,
        tile_id: u64,
    }

    impl TT {
        pub(crate) fn new(wfc_id: u64) -> Self {
            Self {
                pos: (0, 0),
                tile_id: wfc_id,
            }
        }
    }

    impl GridTile2D for TT {
        fn grid_position(&self) -> GridPos2D {
            self.pos
        }

        fn set_grid_position(&mut self, position: GridPos2D) {
            self.pos = position;
        }
    }

    impl IdentifiableTile for TT {
        fn get_tile_id(&self) -> u64 {
            self.tile_id
        }
    }

    pub(crate) type TestMap = GridMap2D<TT>;

    pub(crate) fn get_test_map() -> TestMap {
        let mut tiles = [
            [
                TT::new(0),
                TT::new(0),
                TT::new(1),
                TT::new(1),
                TT::new(0),
                TT::new(0),
            ],
            [
                TT::new(0),
                TT::new(0),
                TT::new(1),
                TT::new(1),
                TT::new(0),
                TT::new(0),
            ],
            [
                TT::new(0),
                TT::new(1),
                TT::new(1),
                TT::new(1),
                TT::new(0),
                TT::new(0),
            ],
            [
                TT::new(1),
                TT::new(1),
                TT::new(2),
                TT::new(2),
                TT::new(0),
                TT::new(1),
            ],
            [
                TT::new(1),
                TT::new(2),
                TT::new(3),
                TT::new(2),
                TT::new(1),
                TT::new(1),
            ],
            [
                TT::new(0),
                TT::new(1),
                TT::new(3),
                TT::new(1),
                TT::new(1),
                TT::new(1),
            ],
        ];

        let mut map = TestMap::new(GridSize::new(6, 6));

        for (y_pos, row) in tiles.iter_mut().enumerate() {
            for (x_pos, tile) in row.iter_mut().enumerate() {
                tile.set_grid_position((x_pos as u32, y_pos as u32));
                map.insert_tile(tile.clone());
            }
        }
        map
    }

    #[test]
    fn can_analyze_map() {
        let map = get_test_map();

        let mut analyzer = WFCAnalyzer::<TT>::default();
        analyzer.analyze(&map);

        assert_eq!(4, analyzer.tiles.len());
    }

    #[test]
    fn can_generate_rules() {
        let map = get_test_map();

        let mut analyzer = WFCAnalyzer::<TT>::default();
        analyzer.analyze(&map);

        let freq = analyzer.frequency();

        assert_eq!(4, freq.get_all_weights_cloned().len());

        let adjacency = analyzer.adjacency();

        assert!(adjacency.is_adjacent_option_valid(1, 2, crate::map::GridDir::RIGHT));
    }
}

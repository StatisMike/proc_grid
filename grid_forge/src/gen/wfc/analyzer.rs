use std::{
    collections::{BTreeMap, BTreeSet},
    marker::PhantomData,
};

use crate::{
    map::{GridDir, GridMap2D},
    GridPos2D,
};

use super::{pattern::WFCNeighbours, WFCTile};

#[derive(Default, Debug, Clone)]
pub struct WFCAnalyzer<T>
where
    T: WFCTile,
{
    tiles: BTreeSet<u64>,
    // tile_funs: BTreeMap<u64, fn(GridPos2D) -> T>,
    freqs: BTreeMap<u64, u32>,
    tiles_count: u32,
    pub(crate) neigbours: BTreeMap<u64, WFCNeighbours>,
    phantom: PhantomData<T>,
}

impl<T> WFCAnalyzer<T>
where
    T: WFCTile,
{
    pub fn new() -> Self {
        Self {
            tiles: BTreeSet::new(),
            // tile_funs: BTreeMap::new(),
            freqs: BTreeMap::new(),
            tiles_count: 0,
            neigbours: BTreeMap::new(),
            phantom: PhantomData::<T>,
        }
    }

    pub fn is_valid_at_dir(&self, option: u64, neighbour_option: u64, dir: &GridDir) -> bool {
      let neighbours = self.neigbours
      .get(&option)
      .unwrap();
    
      neighbours.is_valid_at_dir(dir, neighbour_option)
    }

    pub fn tiles(&self) -> Vec<u64> {
        Vec::from_iter(self.tiles.iter().cloned())
    }

    pub fn analyze(&mut self, map: &GridMap2D<T>) {
        // Iterate on all possible positions
        for position in map.get_all_positions() {
            self.analyze_tile_at_pos(map, position);
        }
    }

    pub fn probs(&self) -> WFCTileProbs {
        WFCTileProbs::new(&self.freqs, self.tiles_count)
    }

    fn analyze_tile_at_pos(&mut self, map: &GridMap2D<T>, pos: GridPos2D) {
        if let Some(tile) = map.get_tile_at_position(&pos) {
            let id = tile.wfc_id();

            for dir in GridDir::ALL {
                if let Some(neighbour) = map.get_neighbour_at(&pos, dir) {
                    if let Some(neighbours) = self.neigbours.get_mut(&id) {
                        neighbours.insert_at_dir(dir, neighbour.wfc_id());
                    } else {
                        let mut neighbours = WFCNeighbours::default();
                        neighbours.insert_at_dir(dir, neighbour.wfc_id());
                        self.neigbours.insert(id, neighbours);
                    }
                }
            }

            if let Some(freqs) = self.freqs.get_mut(&id) {
                *freqs += 1;
            } else {
                self.freqs.insert(id, 1);
            }

            if !self.tiles.contains(&id) {
                self.tiles.insert(id);
            }

            self.tiles_count += 1;
        }
    }
}

#[derive(Debug)]
pub struct WFCTileProbs {
    inner: BTreeMap<u64, f32>,
}

impl WFCTileProbs {
    fn new(freqs: &BTreeMap<u64, u32>, total: u32) -> Self {
        let mut inner = BTreeMap::new();
        for (id, freq) in freqs.iter() {
            inner.insert(*id, *freq as f32 / total as f32);
        }
        Self { inner }
    }

    pub fn tiles_by_prob(&self) -> Vec<u64> {
        let mut tiles_by_prob = self.inner.iter().collect::<Vec<_>>();

        tiles_by_prob.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

        tiles_by_prob
            .iter()
            .map(|x| x.0)
            .copied()
            .collect::<Vec<_>>()
    }

    pub fn wfc_prob(&self, id: u64) -> Option<f32> {
        self.inner.get(&id).copied()
    }

    pub fn total_entropy(&self, ids: &[u64]) -> f32 {
        let all_probs = ids
            .iter()
            .map(|id| self.inner.get(id).copied().unwrap_or_default())
            .filter(|x| x > &0.)
            .collect::<Vec<_>>();

        let sum: f32 = all_probs.iter().sum();

        all_probs
            .iter()
            .map(|x| x / sum)
            .map(|x| -x * x.log2())
            .sum()
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::WFCAnalyzer;
    use crate::{
        gen::wfc::WFCTile,
        map::{GridMap2D, GridSize},
        tile::GridTile2D,
        GridPos2D,
    };

    #[derive(Debug, Clone)]
    pub(crate) struct TT {
        pos: GridPos2D,
        wfc_id: u64,
    }

    impl TT {
        pub(crate) fn new(wfc_id: u64) -> Self {
            Self {
                pos: (0, 0),
                wfc_id,
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

    impl WFCTile for TT {
        fn wfc_id(&self) -> u64 {
            self.wfc_id
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

        let mut consumer = WFCAnalyzer::<TT>::new();
        consumer.analyze(&map);

        assert_eq!(4, consumer.tiles.len());

        assert_eq!(map.size().x() * map.size().y(), consumer.tiles_count);
    }

    #[test]
    fn can_generate_probs() {
        let map = get_test_map();

        let mut consumer = WFCAnalyzer::<TT>::new();
        consumer.analyze(&map);

        let probs = consumer.probs();

        let mut tiles_by_probs = Vec::new();

        for (id, prob) in probs.inner.iter() {
            tiles_by_probs.push((*id, *prob));
        }

        tiles_by_probs.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let higher_entrophy = probs.total_entropy(&[
            tiles_by_probs.first().unwrap().0,
            tiles_by_probs.get(1).unwrap().0,
        ]);
        let lower_entrophy = probs.total_entropy(&[
            tiles_by_probs.first().unwrap().0,
            tiles_by_probs.last().unwrap().0,
        ]);

        assert!(higher_entrophy > lower_entrophy);

        let one_el_entrophy = probs.total_entropy(&[0]);

        assert_eq!(0., one_el_entrophy);
    }
}

use std::{
    collections::{linked_list::Iter, HashMap, HashSet},
    hash::{DefaultHasher, Hash, Hasher},
    marker::PhantomData,
};

use crate::{
    gen::collapse::{tile::CollapsibleData, CollapsibleTileData},
    map::{GridDir, GridMap2D, GridSize},
    tile::{identifiable::IdentifiableTileData, GridPosition, TileContainer},
};

pub type Pattern2D<const PATTERN_WIDTH: usize, const PATTERN_HEIGHT: usize> =
    Pattern<PATTERN_WIDTH, PATTERN_HEIGHT, 1>;

/// Pattern for Overlapping collapse algorithm.
///
/// It describes the tiles present
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Pattern<
    const PATTERN_WIDTH: usize,
    const PATTERN_HEIGHT: usize,
    const PATTERN_DEPTH: usize,
> {
    pattern_id: u64,
    tile_type_id: u64,
    tile_type_ids: [[[u64; PATTERN_WIDTH]; PATTERN_HEIGHT]; PATTERN_DEPTH],
}

impl<const PATTERN_WIDTH: usize, const PATTERN_HEIGHT: usize, const PATTERN_DEPTH: usize>
    Pattern<PATTERN_WIDTH, PATTERN_HEIGHT, PATTERN_DEPTH>
{
    pub(crate) fn new() -> Self {
        Self {
            pattern_id: 0,
            tile_type_id: 0,
            tile_type_ids: [[[0; PATTERN_WIDTH]; PATTERN_HEIGHT]; PATTERN_DEPTH],
        }
    }

    pub(crate) fn is_compatible_tile<Ref: AsRef<CollapsibleTileData> + TileContainer>(
        &self,
        anchor_pos: &GridPosition,
        tile: &Ref,
    ) -> bool {
        if tile.as_ref().is_collapsed() {
            self.get_id_for_pos(anchor_pos, &tile.grid_position()) == tile.as_ref().tile_type_id()
        } else {
            tile.as_ref()
                .options_with_weights
                .contains_key(&self.get_id_for_pos(anchor_pos, &tile.grid_position()))
        }
    }

    fn get_id_for_pos(&self, anchor_pos: &GridPosition, pos: &GridPosition) -> u64 {
        if PATTERN_DEPTH == 1 {
            self.tile_type_ids[0][*anchor_pos.y() as usize - *pos.y() as usize]
                [*anchor_pos.x() as usize - *pos.x() as usize]
        } else {
            self.tile_type_ids[(anchor_pos.z().expect("cannot get `z` from `anchor_pos`")
                - pos.z().expect("cannot get `z` from `pos`"))
                as usize][(anchor_pos.y() - pos.y()) as usize]
                [(anchor_pos.x() - pos.x()) as usize]
        }
    }

    fn set_id_for_pos(&mut self, anchor_pos: &GridPosition, pos: &GridPosition, tile_type_id: u64) {
        if PATTERN_DEPTH == 1 {
            self.tile_type_ids[0][(pos.y() - anchor_pos.y()) as usize]
                [(pos.x() - anchor_pos.x()) as usize] = tile_type_id;
        } else {
            self.tile_type_ids[(pos.z().expect("cannot get `z` from `anchor_pos`")
                - anchor_pos.z().expect("cannot get `z` from `pos`"))
                as usize][(pos.y() - anchor_pos.y()) as usize]
                [(pos.x() - anchor_pos.x()) as usize] = tile_type_id;
        }
    }

    pub fn is_compatible_with(&self, other: &Self, direction: GridDir) -> bool {
        match direction {
            GridDir::UP => self.compare_up(other),
            GridDir::DOWN => self.compare_down(other),
            GridDir::LEFT => self.compare_left(other),
            GridDir::RIGHT => self.compare_right(other),
        }
    }
}

#[derive(Default)]
pub struct OverlappingAnalyzer<Data, Pattern>
where
    Data: IdentifiableTileData,
{
    pattern_counts: HashMap<u64, u32>,
    tile_type: PhantomData<*const Data>,
    patterns: Vec<Pattern>,
}

impl<
        Data: IdentifiableTileData,
        const PATTERN_WIDTH: usize,
        const PATTERN_HEIGHT: usize,
        const PATTERN_DEPTH: usize,
    > OverlappingAnalyzer<Data, Pattern<PATTERN_WIDTH, PATTERN_HEIGHT, PATTERN_DEPTH>>
{
    pub fn new() -> Self {
        Self {
            pattern_counts: HashMap::new(),
            tile_type: PhantomData,
            patterns: Vec::new(),
        }
    }

    pub fn analyze_map(&mut self, map: &GridMap2D<Data>) {
        for position in map.size().get_all_possible_positions() {
            if let Some(pattern) = self.create_pattern(map, &position) {
                if match self.pattern_counts.entry(pattern.pattern_id) {
                    std::collections::hash_map::Entry::Occupied(mut e) => {
                        *e.get_mut() += 1;
                        false
                    }
                    std::collections::hash_map::Entry::Vacant(e) => {
                        e.insert(1);
                        true
                    }
                } {
                    self.patterns.push(pattern);
                }
            }
        }
    }

    fn create_pattern(
        &self,
        map: &GridMap2D<Data>,
        anchor_pos: &GridPosition,
    ) -> Option<Pattern<PATTERN_WIDTH, PATTERN_HEIGHT, PATTERN_DEPTH>> {
        if let Some(positions) = self.generate_pattern_positions(anchor_pos, map.size()) {
            let mut pattern = Pattern::new();
            let tiles = map.get_tiles_at_positions(&positions);
            for tile in tiles {
                pattern.set_id_for_pos(
                    anchor_pos,
                    &tile.grid_position(),
                    tile.as_ref().tile_type_id(),
                );
            }
            pattern.tile_type_id = pattern.tile_type_ids[0][0][0];

            let mut hasher = DefaultHasher::default();
            pattern.tile_type_ids.hash(&mut hasher);
            pattern.pattern_id = hasher.finish();
            return Some(pattern);
        }
        None
    }

    fn generate_pattern_positions(
        &self,
        from: &GridPosition,
        size: &GridSize,
    ) -> Option<Vec<GridPosition>> {
        let mut to = *from;
        to.add_xy(((PATTERN_WIDTH - 1) as u32, (PATTERN_HEIGHT - 1) as u32));
        if !size.is_position_valid(&to) {
            return None;
        }
        Some(GridPosition::generate_rect_area(from, &to))
    }

    pub fn generate_pattern_rules(
        &self,
    ) -> PatternRules<Pattern<PATTERN_WIDTH, PATTERN_HEIGHT, PATTERN_DEPTH>> {
        let mut pattern_by_option: HashMap<
            u64,
            Vec<Pattern<PATTERN_WIDTH, PATTERN_HEIGHT, PATTERN_DEPTH>>,
        > = HashMap::new();
        let mut pattern_weights = HashMap::new();

        for pattern in self.patterns.iter() {
            pattern_weights.insert(
                pattern.pattern_id,
                *self.pattern_counts.get(&pattern.pattern_id).unwrap(),
            );
            match pattern_by_option.entry(pattern.tile_type_id) {
                std::collections::hash_map::Entry::Occupied(mut e) => {
                    e.get_mut().push(pattern.clone());
                }
                std::collections::hash_map::Entry::Vacant(e) => {
                    e.insert(vec![pattern.clone()]);
                }
            }
        }

        PatternRules {
            pattern_by_option,
            pattern_weights,
        }
    }
}

pub struct PatternRules<Pattern> {
    pattern_by_option: HashMap<u64, Vec<Pattern>>,
    pattern_weights: HashMap<u64, u32>,
}

#[cfg(test)]
mod test {
    use super::Pattern;
    use crate::{
        map::{GridDir, GridMap2D, GridSize},
        tile::{
            identifiable::{builders::ConstructableViaIdentifierTile, BasicIdentTileData},
            GridPosition, GridTile,
        },
    };

    use super::{OverlappingAnalyzer, Pattern2D};

    const TEST_4X4: [[u64; 4]; 4] = [[0, 1, 1, 0], [1, 1, 1, 0], [0, 1, 1, 1], [0, 1, 1, 0]];

    fn generate_test_map<const WIDTH: usize, const HEIGHT: usize>(
        tile_ids: &[[u64; WIDTH]; HEIGHT],
    ) -> GridMap2D<BasicIdentTileData> {
        let size = GridSize::new_xy(WIDTH as u32, HEIGHT as u32);

        let mut map = GridMap2D::new(size);

        for (y, row) in tile_ids.iter().enumerate() {
            for (x, tile_type_id) in row.iter().enumerate() {
                map.insert_tile(GridTile::new(
                    GridPosition::new_xy(x as u32, y as u32),
                    BasicIdentTileData::tile_new(*tile_type_id),
                ));
            }
        }
        map
    }

    #[test]
    fn analyze_4x4() {
        let map = generate_test_map(&TEST_4X4);
        let mut analyzer2d = OverlappingAnalyzer::<BasicIdentTileData, Pattern2D<2, 2>>::new();
        analyzer2d.analyze_map(&map);
        println!("{:?}", analyzer2d.pattern_counts);
    }

    #[test]
    fn pattern_compatibility() {
        const WIDTH: usize = 2;
        const HEIGHT: usize = 2;
        const DEPTH: usize = 2;

        let a = Pattern::<WIDTH, HEIGHT, DEPTH> {
            pattern_id: 1,
            tile_type_id: 1,
            tile_type_ids: [[[1, 2], [3, 4]], [[5, 6], [7, 8]]],
        };

        let b = Pattern::<WIDTH, HEIGHT, DEPTH> {
            pattern_id: 2,
            tile_type_id: 2,
            tile_type_ids: [[[3, 4], [1, 2]], [[7, 8], [5, 6]]],
        };

        let direction = GridDir::DOWN;

        let result = a.is_compatible_with(&b, direction);
        println!(
            "Are the patterns compatible in the given direction? {}",
            result
        );
        let direction = GridDir::UP;

        let result = a.is_compatible_with(&b, direction);
        println!(
            "Are the patterns compatible in the given direction? {}",
            result
        );

        let direction = GridDir::LEFT;

        let result = a.is_compatible_with(&b, direction);
        println!(
            "Are the patterns compatible in the given direction? {}",
            result
        );
    }
}

// Pattern compatibility

impl<const PATTERN_WIDTH: usize, const PATTERN_HEIGHT: usize, const PATTERN_DEPTH: usize>
    Pattern<PATTERN_WIDTH, PATTERN_HEIGHT, PATTERN_DEPTH>
{
    fn compare_up(&self, other: &Self) -> bool {
        if PATTERN_HEIGHT == 1 {
            return true;
        }
        for z in 0..PATTERN_DEPTH {
            for y in 1..PATTERN_HEIGHT {
                // Skip y = 0
                for x in 0..PATTERN_WIDTH {
                    if self.tile_type_ids[z][y][x] != other.tile_type_ids[z][y - 1][x] {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn compare_down(&self, other: &Self) -> bool {
        if PATTERN_HEIGHT == 1 {
            return true;
        }
        for z in 0..PATTERN_DEPTH {
            for y in 0..PATTERN_HEIGHT - 1 {
                // Skip y = PATTERN_HEIGHT - 1
                for x in 0..PATTERN_WIDTH {
                    if self.tile_type_ids[z][y][x] != other.tile_type_ids[z][y + 1][x] {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn compare_left(&self, other: &Self) -> bool {
        if PATTERN_WIDTH == 1 {
            return true;
        }
        for z in 0..PATTERN_DEPTH {
            for y in 0..PATTERN_HEIGHT {
                for x in 1..PATTERN_WIDTH {
                    // Skip x = 0
                    if self.tile_type_ids[z][y][x] != other.tile_type_ids[z][y][x - 1] {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn compare_right(&self, other: &Self) -> bool {
        if PATTERN_WIDTH == 1 {
            return true;
        }
        for z in 0..PATTERN_DEPTH {
            for y in 0..PATTERN_HEIGHT {
                for x in 0..PATTERN_WIDTH - 1 {
                    // Skip x = PATTERN_WIDTH - 1
                    if self.tile_type_ids[z][y][x] != other.tile_type_ids[z][y][x + 1] {
                        return false;
                    }
                }
            }
        }
        true
    }
}

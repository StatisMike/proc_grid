use std::{
    collections::{HashMap, HashSet},
    hash::{DefaultHasher, Hash, Hasher},
};

use crate::map::{GridDir, GridMap2D, GridSize};
use crate::tile::{GridPosition, TileContainer, TileData};
use crate::tile::identifiable::collection::IdentTileCollection;
use crate::tile::identifiable::IdentifiableTileData;

pub trait OverlappingPattern: private::Sealed {
    const WIDTH: usize;
    const HEIGHT: usize;
    const DEPTH: usize;

}

/// [Pattern] for two-dimensional grids.
pub type Pattern2D<const P_X: usize, const P_Y: usize> = Pattern<P_X, P_Y, 1>;

/// Pattern for Overlapping collapse algorithm.
///
/// It describes the identifiable tiles present in
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Pattern<const P_X: usize, const P_Y: usize, const P_Z: usize> {
    pub(crate) pattern_id: u64,
    pub(crate) tile_type_id: u64,
    pub(crate) tile_type_ids: [[[u64; P_X]; P_Y]; P_Z],
}

impl<const P_X: usize, const P_Y: usize, const P_Z: usize> Hash for Pattern<P_X, P_Y, P_Z> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.tile_type_ids.hash(state);
    }
}

impl<const P_X: usize, const P_Y: usize, const P_Z: usize> OverlappingPattern
    for Pattern<P_X, P_Y, P_Z>
{
    const WIDTH: usize = P_X;
    const HEIGHT: usize = P_Y;
    const DEPTH: usize = P_Z;
}

impl<const P_X: usize, const P_Y: usize, const P_Z: usize> Pattern<P_X, P_Y, P_Z> {
    pub(crate) fn new() -> Self {
        Self {
            pattern_id: 0,
            tile_type_id: 0,
            tile_type_ids: [[[0; P_X]; P_Y]; P_Z],
        }
    }

    pub(crate) fn get_id_for_pos(&self, anchor_pos: &GridPosition, pos: &GridPosition) -> u64 {
        if P_Z == 1 {
            self.tile_type_ids[0][*anchor_pos.y() as usize - *pos.y() as usize]
                [*anchor_pos.x() as usize - *pos.x() as usize]
        } else {
            self.tile_type_ids[(anchor_pos.z().expect("cannot get `z` from `anchor_pos`")
                - pos.z().expect("cannot get `z` from `pos`"))
                as usize][(anchor_pos.y() - pos.y()) as usize]
                [(anchor_pos.x() - pos.x()) as usize]
        }
    }

    pub(crate) fn set_id_for_pos(
        &mut self,
        anchor_pos: &GridPosition,
        pos: &GridPosition,
        tile_type_id: u64,
    ) {
        if P_Z == 1 {
            self.tile_type_ids[0][(pos.y() - anchor_pos.y()) as usize]
                [(pos.x() - anchor_pos.x()) as usize] = tile_type_id;
        } else {
            self.tile_type_ids[(pos.z().expect("cannot get `z` from `anchor_pos`")
                - anchor_pos.z().expect("cannot get `z` from `pos`"))
                as usize][(pos.y() - anchor_pos.y()) as usize]
                [(pos.x() - anchor_pos.x()) as usize] = tile_type_id;
        }
    }

    pub(crate) fn is_compatible_with(&self, other: &Self, direction: GridDir) -> bool {
        match direction {
            GridDir::UP => self.compare_up(other),
            GridDir::DOWN => self.compare_down(other),
            GridDir::LEFT => self.compare_left(other),
            GridDir::RIGHT => self.compare_right(other),
        }
    }

    pub(crate) fn other_tile_positions(anchor_pos: &GridPosition) -> Vec<GridPosition> {
        let mut out = Vec::new();
        for x_off in 0..P_X {
            for y_off in 0..P_Y {
                for z_off in 0..P_Z {
                    if x_off == 0 && y_off == 0 && z_off == 0 {
                        continue;
                    }
                    out.push({
                        let mut pos = *anchor_pos;
                        pos.add_xy((x_off as u32, y_off as u32));
                        pos.add_z(z_off as u32);
                        pos
                    })
                }
            }
        }
        out
    }

    // ------ Comparison methods ------ //
    fn compare_up(&self, other: &Self) -> bool {
        if P_Y == 1 {
            return true;
        }
        for z in 0..P_Z {
            for y in 0..P_Y - 1 {
                for x in 0..P_X {
                    if self.tile_type_ids[z][y][x] != other.tile_type_ids[z][y + 1][x] {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn compare_down(&self, other: &Self) -> bool {
        if P_Y == 1 {
            return true;
        }
        for z in 0..P_Z {
            for y in 1..P_Y {
                for x in 0..P_X {
                    if self.tile_type_ids[z][y][x] != other.tile_type_ids[z][y - 1][x] {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn compare_left(&self, other: &Self) -> bool {
        if P_X == 1 {
            return true;
        }
        for z in 0..P_Z {
            for y in 0..P_Y {
                for x in 0..P_X - 1 {
                    if self.tile_type_ids[z][y][x] != other.tile_type_ids[z][y][x + 1] {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn compare_right(&self, other: &Self) -> bool {
        if P_X == 1 {
            return true;
        }
        for z in 0..P_Z {
            for y in 0..P_Y {
                for x in 1..P_X {
                    if self.tile_type_ids[z][y][x] != other.tile_type_ids[z][y][x - 1] {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn compare_deeper(&self, other: &Self) -> bool {
        if P_Z == 1 {
            return true;
        }
        for z in 0..P_Z - 1 {
            for y in 0..P_Y {
                for x in 0..P_X {
                    if self.tile_type_ids[z][y][x] != other.tile_type_ids[z + 1][y][x] {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn compare_shallower(&self, other: &Self) -> bool {
        if P_Z == 1 {
            return true;
        }
        for z in 1..P_Z {
            for y in 0..P_Y {
                for x in 0..P_X {
                    if self.tile_type_ids[z][y][x] != other.tile_type_ids[z - 1][y][x] {
                        return false;
                    }
                }
            }
        }
        true
    }
}

#[derive(Debug, Default, Clone)]
/// Collection holding all found patterns.
pub struct PatternCollection<const P_X: usize, const P_Y: usize, const P_Z: usize> {
    inner: HashMap<u64, Pattern<P_X, P_Y, P_Z>>,
    rev: HashMap<u64, u64>,
    by_tile_id: HashMap<u64, HashSet<u64>>,
}

impl<const P_X: usize, const P_Y: usize, const P_Z: usize> PatternCollection<P_X, P_Y, P_Z> {
    pub fn get_patterns_for_tile(&self, tile_type_id: u64) -> Vec<&Pattern<P_X, P_Y, P_Z>> {
        if let Some(patterns) = self.by_tile_id.get(&tile_type_id) {
            patterns
                .iter()
                .filter_map(|pattern_id| self.inner.get(pattern_id))
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    }
}

impl<const P_X: usize, const P_Y: usize, const P_Z: usize> IdentTileCollection
    for PatternCollection<P_X, P_Y, P_Z>
{
    type DATA = Pattern<P_X, P_Y, P_Z>;

    fn inner(&self) -> &std::collections::HashMap<u64, Self::DATA> {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut std::collections::HashMap<u64, Self::DATA> {
        &mut self.inner
    }

    fn rev(&self) -> &std::collections::HashMap<u64, u64> {
        &self.rev
    }

    fn rev_mut(&mut self) -> &mut std::collections::HashMap<u64, u64> {
        &mut self.rev
    }

    fn on_add(&mut self, data: &Self::DATA) {
        match self.by_tile_id.entry(data.tile_type_id) {
            std::collections::hash_map::Entry::Occupied(mut e) => {
                e.get_mut().insert(data.pattern_id);
            }
            std::collections::hash_map::Entry::Vacant(e) => {
                e.insert(HashSet::from_iter([data.pattern_id]));
            }
        }
    }

    fn on_remove(&mut self, data: &Self::DATA) {
        if let Some(set) = self.by_tile_id.get_mut(&data.tile_type_id) {
            set.remove(&data.pattern_id);
        }
    }
}

/// Tile data of inner grid within [`OverlappingPatternGrid`].
#[derive(Debug, Clone)]
pub enum PatternTileData {
    /// Tile which besides containing information about `tile_type_id` of the original [`IdentifiableTileData`],
    /// is also a first tile of pattern with given `pattern_id` (so it is in position `[0][0][0]` within the pattern).
    WithPattern { tile_type_id: u64, pattern_id: u64 },
    /// Tile which contains only information about `tile_type_id` of original [`IdentifiableTileData`]. No pattern
    /// information is held, as its position makes it impossible to be first tile of any pattern.
    OnlyId { tile_type_id: u64 },
}

impl TileData for PatternTileData {}

/// Grid containing pattern data derived from original [`GridMap2D`].
#[derive(Debug, Clone)]
pub struct OverlappingPatternGrid<const P_X: usize, const P_Y: usize, const P_Z: usize> {
    inner: GridMap2D<PatternTileData>,
}

impl<const P_X: usize, const P_Y: usize, const P_Z: usize> OverlappingPatternGrid<P_X, P_Y, P_Z> {
    /// Prepare new instance out of [`GridMap2D`], populating provided [`PatternCollection`] in the process.
    pub fn from_map<Data: IdentifiableTileData>(
        map: &GridMap2D<Data>,
        collection: &mut PatternCollection<P_X, P_Y, P_Z>,
    ) -> Self {
        let mut instance = Self {
            inner: GridMap2D::new(*map.size()),
        };

        for position in map.get_all_positions() {
            if let Some(pattern) = instance.create_pattern(map, &position) {
                let tile = PatternTileData::WithPattern {
                    tile_type_id: pattern.tile_type_id,
                    pattern_id: pattern.pattern_id,
                };
                collection.add_tile(pattern);
                instance.inner.insert_data(&position, tile);
            } else if let Some(ident_tile) = map.get_tile_at_position(&position) {
                let tile = PatternTileData::OnlyId {
                    tile_type_id: ident_tile.as_ref().tile_type_id(),
                };
                instance.inner.insert_data(&position, tile);
            }
        }

        instance
    }

    /// Gets a reference to inner [`GridMap2D`] containing [`PatternTileData`].
    ///
    /// Useful for getting insight into pattern extracted from specific portion of original map.
    pub fn inner(&self) -> &GridMap2D<PatternTileData> {
        &self.inner
    }

    fn create_pattern<Data: IdentifiableTileData>(
        &self,
        map: &GridMap2D<Data>,
        anchor_pos: &GridPosition,
    ) -> Option<Pattern<P_X, P_Y, P_Z>> {
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
            pattern.hash(&mut hasher);
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
        to.add_xy(((P_X - 1) as u32, (P_Y - 1) as u32));
        if !size.is_position_valid(&to) {
            return None;
        }
        Some(GridPosition::generate_rect_area(from, &to))
    }
}

mod private {
    use super::*;

    pub trait Sealed {}

    impl<const P_X: usize, const P_Y: usize, const P_Z: usize> Sealed for Pattern<P_X, P_Y, P_Z> {}
}

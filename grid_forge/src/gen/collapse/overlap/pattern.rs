use std::collections::{HashMap, HashSet};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::marker::PhantomData;

use crate::map::{GridDir, GridMap2D, GridSize};
use crate::tile::identifiable::collection::IdentTileCollection;
use crate::tile::identifiable::IdentifiableTileData;
use crate::tile::{GridPosition, TileContainer, TileData};

/// Pattern used in Overlapping Collapse algorithm.
///
/// To declare the size of pattern to use, specify the dimensionality by providing [`OverlappingPattern2D`] or
/// [`OverlappingPattern3D`] type signature in the dependent type declarations. Sizes of 2 and 3 tiles are the most
/// efficient and provide most interesting outputs, though it all depends of the structute in the input grids.
///
/// Specifying a size of `1` in all directions will provide totally random outputs. For the single-tiled collapsible
/// generative algorithm the [`adjacency`](crate::gen::collapse::adjacency)-based should be always preferred.
///
/// It is comprised of `tile_type_ids` of all tiles found in the range of the pattern, where the one present at `[0][0][0]`
/// position is considered the *main* tile, and all others are *secondary* tiles.
///
/// Algorithm selecting a pattern to be present in some place will result in the *main* tile to be placed there, while
/// *secondary* tiles are there to check compatibility beetween two different `OverlappingPattern`s.
#[allow(private_bounds)]
pub trait OverlappingPattern
where
    Self: Clone + PartialEq + Eq + Hash + std::fmt::Debug + private::SealedPattern,
{
    /// Size of the pattern on the `x` axis.
    const X_LEN: usize;

    /// Size of the pattern on the `y` axis.
    const Y_LEN: usize;

    /// Size of the pattern on the `z` axis.
    const Z_LEN: usize;

    /// Retrieves pattern identifier.
    fn pattern_id(&self) -> u64;

    /// Retrieves `tile_type_id` of pattern primary tile.
    fn tile_type_id(&self) -> u64;

    /// Gets `tile_type_id` for a [`TileData`] of a tile present in the pattern, given the [`GridPosition`] of the
    /// primary tile (`anchor_pos`) and specific position (`pos`).
    ///
    /// # Panic
    /// This method will panic if the `pos` is located beyond boundaries of the pattern.
    fn get_id_for_pos(&self, anchor_pos: &GridPosition, pos: &GridPosition) -> u64;

    /// Checks compatibility between two patterns is specified direction.
    fn is_compatible_with(&self, other: &Self, direction: GridDir) -> bool;

    /// Retrieves positions of the secondary tiles of the pattern.
    fn secondary_tile_positions(anchor_pos: &GridPosition) -> Vec<GridPosition>;
}

/// [OverlappingPattern] for two-dimensional grids.
pub type OverlappingPattern2D<const X_LEN: usize, const Y_LEN: usize> =
    OverlappingPattern3D<X_LEN, Y_LEN, 1>;

/// [OverlappingPattern] for three-dimensional grids.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OverlappingPattern3D<const X_LEN: usize, const Y_LEN: usize, const Z_LEN: usize> {
    pattern_id: u64,
    tile_type_id: u64,
    tile_type_ids: [[[u64; X_LEN]; Y_LEN]; Z_LEN],
}

impl<const P_X: usize, const P_Y: usize, const P_Z: usize> Hash
    for OverlappingPattern3D<P_X, P_Y, P_Z>
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.tile_type_ids.hash(state);
    }
}

impl<const P_X: usize, const P_Y: usize, const P_Z: usize> OverlappingPattern
    for OverlappingPattern3D<P_X, P_Y, P_Z>
{
    const X_LEN: usize = P_X;
    const Y_LEN: usize = P_Y;
    const Z_LEN: usize = P_Z;

    fn pattern_id(&self) -> u64 {
        self.pattern_id
    }

    fn tile_type_id(&self) -> u64 {
        self.tile_type_id
    }

    fn get_id_for_pos(&self, anchor_pos: &GridPosition, pos: &GridPosition) -> u64 {
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

    fn is_compatible_with(&self, other: &Self, direction: GridDir) -> bool {
        match direction {
            GridDir::UP => self.compare_up(other),
            GridDir::DOWN => self.compare_down(other),
            GridDir::LEFT => self.compare_left(other),
            GridDir::RIGHT => self.compare_right(other),
        }
    }

    fn secondary_tile_positions(anchor_pos: &GridPosition) -> Vec<GridPosition> {
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
}

impl<const P_X: usize, const P_Y: usize, const P_Z: usize> OverlappingPattern3D<P_X, P_Y, P_Z> {
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

#[derive(Debug, Clone)]
/// Collection holding all found patterns.
pub struct PatternCollection<P: OverlappingPattern> {
    inner: HashMap<u64, P>,
    rev: HashMap<u64, u64>,
    by_tile_id: HashMap<u64, HashSet<u64>>,
}

impl<P: OverlappingPattern> Default for PatternCollection<P> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            rev: Default::default(),
            by_tile_id: Default::default(),
        }
    }
}

impl<P: OverlappingPattern> PatternCollection<P> {
    pub fn get_patterns_for_tile(&self, tile_type_id: u64) -> Vec<&P> {
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

impl<P: OverlappingPattern> IdentTileCollection for PatternCollection<P> {
    type DATA = P;

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
        match self.by_tile_id.entry(data.tile_type_id()) {
            std::collections::hash_map::Entry::Occupied(mut e) => {
                e.get_mut().insert(data.pattern_id());
            }
            std::collections::hash_map::Entry::Vacant(e) => {
                e.insert(HashSet::from_iter([data.pattern_id()]));
            }
        }
    }

    fn on_remove(&mut self, data: &Self::DATA) {
        if let Some(set) = self.by_tile_id.get_mut(&data.tile_type_id()) {
            set.remove(&data.pattern_id());
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
pub struct OverlappingPatternGrid<P: OverlappingPattern> {
    inner: GridMap2D<PatternTileData>,
    phantom: PhantomData<fn(P)>,
}

impl<P: OverlappingPattern> OverlappingPatternGrid<P> {
    /// Prepare new instance out of [`GridMap2D`], populating provided [`PatternCollection`] in the process.
    pub fn from_map<Data: IdentifiableTileData>(
        map: &GridMap2D<Data>,
        collection: &mut PatternCollection<P>,
    ) -> Self {
        let mut instance = Self {
            inner: GridMap2D::new(*map.size()),
            phantom: PhantomData,
        };

        for position in map.get_all_positions() {
            if let Some(pattern) = instance.create_pattern(map, &position) {
                let tile = PatternTileData::WithPattern {
                    tile_type_id: pattern.tile_type_id(),
                    pattern_id: pattern.pattern_id(),
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
    ) -> Option<P> {
        if let Some(positions) = self.generate_pattern_positions(anchor_pos, map.size()) {
            let mut pattern = P::empty();
            let tiles = map.get_tiles_at_positions(&positions);
            for tile in tiles {
                pattern.set_id_for_pos(
                    anchor_pos,
                    &tile.grid_position(),
                    tile.as_ref().tile_type_id(),
                );
            }
            pattern.finalize();
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
        to.add_xy(((P::X_LEN - 1) as u32, (P::Y_LEN - 1) as u32));
        if !size.is_position_valid(&to) {
            return None;
        }
        Some(GridPosition::generate_rect_area(from, &to))
    }
}

mod private {
    use super::*;

    /// Trait making the [`OverlappingPattern`] non-implementable outside of the crate and keeping the mutability
    /// methods private to the crate.
    pub(crate) trait SealedPattern {
        fn empty() -> Self;

        fn set_id_for_pos(
            &mut self,
            anchor_pos: &GridPosition,
            pos: &GridPosition,
            tile_type_id: u64,
        );

        fn finalize(&mut self);
    }

    impl<const P_X: usize, const P_Y: usize, const P_Z: usize> SealedPattern
        for OverlappingPattern3D<P_X, P_Y, P_Z>
    {
        fn empty() -> Self {
            Self {
                pattern_id: 0,
                tile_type_id: 0,
                tile_type_ids: [[[0; P_X]; P_Y]; P_Z],
            }
        }

        fn set_id_for_pos(
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

        fn finalize(&mut self) {
            let mut hasher = DefaultHasher::default();
            self.hash(&mut hasher);
            self.pattern_id = hasher.finish();
            self.tile_type_id = self.tile_type_ids[0][0][0];
        }
    }
}

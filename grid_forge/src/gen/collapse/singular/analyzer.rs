use std::collections::{BTreeMap, HashMap};
use std::marker::PhantomData;

use crate::gen::collapse::AdjacencyTable;
use crate::map::{GridDir, GridMap2D};
use crate::tile::identifiable::IdentifiableTileData;
use crate::tile::{GridPosition, TileContainer};

use super::tile;

pub trait Analyzer<Data>
where
    Data: IdentifiableTileData,
{
    fn adjacency(&self) -> &AdjacencyRules<Data>;
    fn tiles(&self) -> &[u64];
    fn analyze(&mut self, map: &GridMap2D<Data>);
}

#[derive(Debug)]
pub struct AdjacencyRules<Data>
where
    Data: IdentifiableTileData,
{
    inner: AdjacencyTable,
    id_type: PhantomData<Data>,
}

impl<Data> Clone for AdjacencyRules<Data>
where
    Data: IdentifiableTileData,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            id_type: PhantomData::<Data>,
        }
    }
}

impl<Data> Default for AdjacencyRules<Data>
where
    Data: IdentifiableTileData,
{
    fn default() -> Self {
        Self {
            inner: AdjacencyTable::default(),
            id_type: PhantomData::<Data>,
        }
    }
}

impl<Data> AdjacencyRules<Data>
where
    Data: IdentifiableTileData,
{
    fn add_adjacency<Tile: AsRef<Data>>(
        &mut self,
        tile: &Tile,
        adjacent_tile: &Tile,
        direction: GridDir,
    ) {
        self.add_adjacency_raw(
            tile.as_ref().tile_type_id(),
            adjacent_tile.as_ref().tile_type_id(),
            direction,
        )
    }

    pub(crate) fn add_adjacency_raw(&mut self, tile_id: u64, adjacent_id: u64, direction: GridDir) {
        self.inner.insert_adjacency(tile_id, direction, adjacent_id);
    }

    pub(crate) fn is_valid_raw(&self, tile_id: u64, adjacent_id: u64, direction: GridDir) -> bool {
        self.inner
            .check_adjacency(&tile_id, &direction, &adjacent_id)
    }

    pub(crate) fn is_valid_raw_any(
        &self,
        tile_id: u64,
        adjacent_options: &[u64],
        direction: GridDir,
    ) -> bool {
        self.inner
            .check_adjacency_any(&tile_id, &direction, adjacent_options)
    }

    pub(crate) fn inner(&self) -> &AdjacencyTable {
        &self.inner
    }
}

pub struct IdentityAnalyzer<Data>
where
    Data: IdentifiableTileData,
{
    tiles: Vec<u64>,
    adjacency_rules: AdjacencyRules<Data>,
}

impl<Data> Default for IdentityAnalyzer<Data>
where
    Data: IdentifiableTileData,
{
    fn default() -> Self {
        Self {
            tiles: Vec::new(),
            adjacency_rules: AdjacencyRules::default(),
        }
    }
}

impl<Data> IdentityAnalyzer<Data>
where
    Data: IdentifiableTileData,
{
    fn analyze_tile_at_pos(&mut self, map: &GridMap2D<Data>, pos: GridPosition) {
        if let Some(tile) = map.get_tile_at_position(&pos) {
            if !self.tiles.contains(&tile.as_ref().tile_type_id()) {
                self.tiles.push(tile.as_ref().tile_type_id());
            }

            for dir in GridDir::ALL_2D {
                if let Some(neighbour) = map.get_neighbour_at(&pos, dir) {
                    self.adjacency_rules.add_adjacency(&tile, &neighbour, *dir)
                }
            }
        }
    }

    pub fn adjacency(&self) -> &AdjacencyRules<Data> {
        &self.adjacency_rules
    }
}

impl<Data> Analyzer<Data> for IdentityAnalyzer<Data>
where
    Data: IdentifiableTileData,
{
    fn analyze(&mut self, map: &GridMap2D<Data>) {
        for position in map.get_all_positions() {
            self.analyze_tile_at_pos(map, position);
        }
    }

    fn adjacency(&self) -> &AdjacencyRules<Data> {
        &self.adjacency_rules
    }

    fn tiles(&self) -> &[u64] {
        &self.tiles
    }
}

pub struct BorderAnalyzer<Data>
where
    Data: IdentifiableTileData,
{
    tiles: Vec<u64>,
    adjacency_rules: AdjacencyRules<Data>,
    /// TileId key
    inner: HashMap<u64, TileBordersAdjacency<Data>>,
    /// BorderId key; (TileId; GridDir)
    border_types: HashMap<u64, Vec<(u64, GridDir)>>,
}

impl<Data> Default for BorderAnalyzer<Data>
where
    Data: IdentifiableTileData,
{
    fn default() -> Self {
        Self {
            tiles: Vec::new(),
            adjacency_rules: AdjacencyRules::default(),
            inner: HashMap::new(),
            border_types: HashMap::new(),
        }
    }
}

impl<Data> Analyzer<Data> for BorderAnalyzer<Data>
where
    Data: IdentifiableTileData,
{
    fn analyze(&mut self, map: &GridMap2D<Data>) {
        self.adjacency_rules = AdjacencyRules::default();
        for position in map.get_all_positions() {
            self.analyze_tile_at_pos(map, position);
        }
        self.generate_adjacency_rules();
    }

    fn adjacency(&self) -> &AdjacencyRules<Data> {
        &self.adjacency_rules
    }

    fn tiles(&self) -> &[u64] {
        &self.tiles
    }
}

impl<Data> BorderAnalyzer<Data>
where
    Data: IdentifiableTileData,
{
    pub fn add_adjacency(&mut self, tile: &Data, neighbour: &Data, direction: &GridDir) {
        self.add_adjacency_raw(tile.tile_type_id(), neighbour.tile_type_id(), direction)
    }

    pub fn prepare(&mut self) {
        self.generate_adjacency_rules()
    }

    fn analyze_tile_at_pos(&mut self, map: &GridMap2D<Data>, pos: GridPosition) {
        if let Some(tile) = map.get_tile_at_position(&pos) {
            if !self.tiles.contains(&tile.as_ref().tile_type_id()) {
                self.tiles.push(tile.as_ref().tile_type_id());
            }

            for dir in GridDir::ALL_2D {
                if let Some(neighbour) = map.get_neighbour_at(&pos, dir) {
                    self.add_adjacency_raw(
                        tile.as_ref().tile_type_id(),
                        neighbour.as_ref().tile_type_id(),
                        dir,
                    );
                }
            }
        }
    }

    fn generate_adjacency_rules(&mut self) {
        let border_ids = self.border_types.keys().copied().collect::<Vec<_>>();

        for border_id in border_ids.iter() {
            let borders = self.border_types.get(border_id).unwrap().clone();

            for half_dir in [GridDir::LEFT, GridDir::UP] {
                let first_borders = borders
                    .iter()
                    .filter_map(
                        |(tile, dir)| {
                            if *dir == half_dir {
                                Some(*tile)
                            } else {
                                None
                            }
                        },
                    )
                    .collect::<Vec<_>>();
                let second_borders = borders
                    .iter()
                    .filter_map(|(tile, dir)| {
                        if *dir == half_dir.opposite() {
                            Some(*tile)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                for tile_first in first_borders.iter() {
                    for tile_second in second_borders.iter() {
                        self.adjacency_rules
                            .add_adjacency_raw(*tile_first, *tile_second, half_dir);
                        self.adjacency_rules.add_adjacency_raw(
                            *tile_second,
                            *tile_first,
                            half_dir.opposite(),
                        );
                    }
                }
            }
        }
    }

    pub(crate) fn add_adjacency_raw(
        &mut self,
        tile_id: u64,
        adjacent_id: u64,
        direction: &GridDir,
    ) {
        self.ensure_adjacencies_present_for_tiles(&[tile_id, adjacent_id]);

        match (
            self.get_border_id(&tile_id, direction),
            self.get_border_id(&adjacent_id, &direction.opposite()),
        ) {
            (None, None) => {
                let new_id = self.get_next_border_id();
                self.set_border_id(new_id, tile_id, direction);
                self.set_border_id(new_id, adjacent_id, &direction.opposite());
            }
            (None, Some(id_border)) => {
                self.set_border_id(id_border, tile_id, direction);
            }
            (Some(id_border), None) => {
                self.set_border_id(id_border, adjacent_id, &direction.opposite());
            }
            (Some(id_left), Some(id_right)) => {
                if id_left == id_right {
                    return;
                }
                self.unify_border_id(id_left.max(id_right), id_left.min(id_right));
            }
        }
    }

    fn ensure_adjacencies_present_for_tiles(&mut self, ids: &[u64]) {
        for id in ids {
            if !self.inner.contains_key(id) {
                self.inner.insert(*id, TileBordersAdjacency::default());
            }
        }
    }

    fn set_border_id(&mut self, border_id: u64, tile_id: u64, direction: &GridDir) {
        self.inner
            .get_mut(&tile_id)
            .unwrap()
            .set_at_dir(direction, border_id);
        self.border_types
            .entry(border_id)
            .or_default()
            .push((tile_id, *direction));
    }

    fn get_border_id(&self, tile_id: &u64, direction: &GridDir) -> Option<u64> {
        self.inner
            .get(tile_id)
            .unwrap()
            .get_at_dir(direction)
            .copied()
    }

    fn unify_border_id(&mut self, existing: u64, into: u64) {
        let cache = self.border_types.remove(&existing).unwrap();
        for (tile_id, direction) in cache.iter() {
            self.set_border_id(into, *tile_id, direction);
        }
    }

    fn get_next_border_id(&self) -> u64 {
        if let Some(max_id) = self.border_types.keys().max() {
            *max_id + 1
        } else {
            0
        }
    }
}

// -------- PRIVATE -------- //

#[derive(Debug, Clone)]
struct InnerAdjacency {
    ia: HashMap<GridDir, Vec<u64>>,
}

impl Default for InnerAdjacency {
    fn default() -> Self {
        let mut ia = HashMap::new();
        for dir in GridDir::ALL_2D {
            ia.insert(*dir, Vec::new());
        }
        Self { ia }
    }
}

impl InnerAdjacency {
    fn add_option(&mut self, adjacent_id: u64, dir: GridDir) {
        let opts = self.ia.get_mut(&dir).unwrap();
        if !opts.contains(&adjacent_id) {
            opts.push(adjacent_id);
        }
    }

    fn is_in_options(&self, adjacent_id: u64, dir: GridDir) -> bool {
        self.ia.get(&dir).unwrap().contains(&adjacent_id)
    }

    fn is_in_options_any(&self, adjacent_options: &[u64], dir: GridDir) -> bool {
        let options = self.ia.get(&dir).unwrap();
        adjacent_options.iter().any(|opt| options.contains(opt))
    }
}

struct TileBordersAdjacency<Data>
where
    Data: IdentifiableTileData,
{
    borders: HashMap<GridDir, u64>,
    phantom: PhantomData<Data>,
}

impl<Data> TileBordersAdjacency<Data>
where
    Data: IdentifiableTileData,
{
    fn set_at_dir(&mut self, dir: &GridDir, border_id: u64) {
        self.borders.insert(*dir, border_id);
    }

    fn get_at_dir(&self, dir: &GridDir) -> Option<&u64> {
        self.borders.get(dir)
    }
}

impl<Data> Default for TileBordersAdjacency<Data>
where
    Data: IdentifiableTileData,
{
    fn default() -> Self {
        Self {
            borders: HashMap::new(),
            phantom: PhantomData::<Data>,
        }
    }
}

/// Frequency hints for the *adjacency-based* generative algorithm.
///
/// Describes the frequency of occurence of all distinct tiles. Can be generated automatically while analyzing sample
/// maps, or specified manually for each `tile_type_id` via [`set_weight_for_tile`](Self::set_weight_for_tile) method.
#[derive(Debug)]
pub struct FrequencyHints<Data>
where
    Data: IdentifiableTileData,
{
    weights: BTreeMap<u64, u32>,
    id_type: PhantomData<Data>,
}

impl<Data> Clone for FrequencyHints<Data>
where
    Data: IdentifiableTileData,
{
    fn clone(&self) -> Self {
        Self {
            weights: self.weights.clone(),
            id_type: PhantomData::<Data>,
        }
    }
}

impl<T> Default for FrequencyHints<T>
where
    T: IdentifiableTileData,
{
    fn default() -> Self {
        Self {
            weights: BTreeMap::new(),
            id_type: PhantomData::<T>,
        }
    }
}

impl<Data> FrequencyHints<Data>
where
    Data: IdentifiableTileData,
{
    pub fn set_weight_for_tile<Tile>(&mut self, tile: &Tile, weight: u32)
    where
        Tile: TileContainer + AsRef<Data>,
    {
        let entry = self
            .weights
            .entry(tile.as_ref().tile_type_id())
            .or_default();
        *entry = weight;
    }

    pub fn count_tile<Tile>(&mut self, tile: &Tile)
    where
        Tile: TileContainer + AsRef<Data>,
    {
        if let Some(count) = self.weights.get_mut(&tile.as_ref().tile_type_id()) {
            *count += 1;
        } else {
            self.weights.insert(tile.as_ref().tile_type_id(), 1);
        }
    }

    pub(crate) fn get_all_weights_cloned(&self) -> BTreeMap<u64, u32> {
        self.weights.clone()
    }

    pub fn analyze_grid_map(&mut self, map: &GridMap2D<Data>) {
        for position in map.get_all_positions() {
            let reference = map.get_tile_at_position(&position).unwrap();
            self.count_tile(&reference)
        }
    }
}

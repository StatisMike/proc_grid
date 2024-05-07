use std::{collections::HashMap, marker::PhantomData};

use crate::{
    map::{GridDir, GridMap2D},
    tile::identifiable::IdentifiableTile,
    GridPos2D,
};

pub trait AdjacencyAnalyzer<T>
where
    T: IdentifiableTile,
{
    fn adjacency(&self) -> &AdjacencyRules<T>;
    fn tiles(&self) -> &[u64];
    fn analyze(&mut self, map: &GridMap2D<T>);
}

#[derive(Debug)]
pub struct AdjacencyRules<T>
where
    T: IdentifiableTile,
{
    inner: HashMap<u64, InnerAdjacency>,
    id_type: PhantomData<T>,
}

impl<T> Clone for AdjacencyRules<T>
where
    T: IdentifiableTile,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            id_type: PhantomData::<T>,
        }
    }
}

impl<T> Default for AdjacencyRules<T>
where
    T: IdentifiableTile,
{
    fn default() -> Self {
        Self {
            inner: HashMap::new(),
            id_type: PhantomData::<T>,
        }
    }
}

impl<T> AdjacencyRules<T>
where
    T: IdentifiableTile,
{
    fn add_adjacency(&mut self, tile: &T, adjacent_tile: &T, direction: GridDir) {
        self.add_adjacency_raw(tile.tile_type_id(), adjacent_tile.tile_type_id(), direction)
    }

    pub(crate) fn add_adjacency_raw(&mut self, tile_id: u64, adjacent_id: u64, direction: GridDir) {
        let adjacents = self.inner.entry(tile_id).or_default();

        adjacents.add_option(adjacent_id, direction);
    }

    pub(crate) fn is_valid_raw(&self, tile_id: u64, adjacent_id: u64, direction: GridDir) -> bool {
        if let Some(rules) = self.inner.get(&tile_id) {
            rules.is_in_options(adjacent_id, direction)
        } else {
            false
        }
    }

    pub(crate) fn is_valid_raw_any(
        &self,
        tile_id: u64,
        adjacent_options: &[u64],
        direction: GridDir,
    ) -> bool {
        if let Some(rules) = self.inner.get(&tile_id) {
            rules.is_in_options_any(adjacent_options, direction)
        } else {
            false
        }
    }
}

pub struct AdjacencyIdentityAnalyzer<T>
where
    T: IdentifiableTile,
{
    tiles: Vec<u64>,
    adjacency_rules: AdjacencyRules<T>,
}

impl<T> Default for AdjacencyIdentityAnalyzer<T>
where
    T: IdentifiableTile,
{
    fn default() -> Self {
        Self {
            tiles: Vec::new(),
            adjacency_rules: AdjacencyRules::default(),
        }
    }
}

impl<T> AdjacencyIdentityAnalyzer<T>
where
    T: IdentifiableTile,
{
    fn analyze_tile_at_pos(&mut self, map: &GridMap2D<T>, pos: GridPos2D) {
        if let Some(tile) = map.get_tile_at_position(&pos) {
            if !self.tiles.contains(&tile.tile_type_id()) {
                self.tiles.push(tile.tile_type_id());
            }

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
}

impl<T> AdjacencyAnalyzer<T> for AdjacencyIdentityAnalyzer<T>
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

    fn tiles(&self) -> &[u64] {
        &self.tiles
    }
}

pub struct AdjacencyBorderAnalyzer<T>
where
    T: IdentifiableTile,
{
    tiles: Vec<u64>,
    adjacency_rules: AdjacencyRules<T>,
    /// TileId key
    inner: HashMap<u64, TileBordersAdjacency<T>>,
    /// BorderId key; (TileId; GridDir)
    border_types: HashMap<u64, Vec<(u64, GridDir)>>,
}

impl<T> Default for AdjacencyBorderAnalyzer<T>
where
    T: IdentifiableTile,
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

impl<T> AdjacencyAnalyzer<T> for AdjacencyBorderAnalyzer<T>
where
    T: IdentifiableTile,
{
    fn analyze(&mut self, map: &GridMap2D<T>) {
        self.adjacency_rules = AdjacencyRules::default();
        for position in map.get_all_positions() {
            self.analyze_tile_at_pos(map, position);
        }
        self.generate_adjacency_rules();
    }

    fn adjacency(&self) -> &AdjacencyRules<T> {
        &self.adjacency_rules
    }

    fn tiles(&self) -> &[u64] {
        &self.tiles
    }
}

impl<T> AdjacencyBorderAnalyzer<T>
where
    T: IdentifiableTile,
{
    pub fn add_adjacency(&mut self, tile: &T, neighbour: &T, direction: &GridDir) {
        self.add_adjacency_raw(tile.tile_type_id(), neighbour.tile_type_id(), direction)
    }

    pub fn prepare(&mut self) {
        self.generate_adjacency_rules()
    }

    fn analyze_tile_at_pos(&mut self, map: &GridMap2D<T>, pos: GridPos2D) {
        if let Some(tile) = map.get_tile_at_position(&pos) {
            if !self.tiles.contains(&tile.tile_type_id()) {
                self.tiles.push(tile.tile_type_id());
            }

            for dir in GridDir::ALL {
                if let Some(neighbour) = map.get_neighbour_at(&pos, dir) {
                    self.add_adjacency_raw(tile.tile_type_id(), neighbour.tile_type_id(), dir);
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
        for dir in GridDir::ALL {
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

struct TileBordersAdjacency<T>
where
    T: IdentifiableTile,
{
    borders: HashMap<GridDir, u64>,
    phantom: PhantomData<T>,
}

impl<T> TileBordersAdjacency<T>
where
    T: IdentifiableTile,
{
    fn set_at_dir(&mut self, dir: &GridDir, border_id: u64) {
        self.borders.insert(*dir, border_id);
    }

    fn get_at_dir(&self, dir: &GridDir) -> Option<&u64> {
        self.borders.get(dir)
    }
}

impl<T> Default for TileBordersAdjacency<T>
where
    T: IdentifiableTile,
{
    fn default() -> Self {
        Self {
            borders: HashMap::new(),
            phantom: PhantomData::<T>,
        }
    }
}

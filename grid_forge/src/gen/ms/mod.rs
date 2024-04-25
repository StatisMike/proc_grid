use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
};

use crate::{
    map::{GridDir, GridMap2D},
    GridPos2D,
};

use super::{
    adjacency::{AdjacencyAnalyzer, AdjacencyRules, IdentifiableTile},
    frequency::FrequencyRules,
};

pub struct MSAnalyzer<T>
where
    T: IdentifiableTile,
{
    tiles: HashSet<u64>,
    frequency_rules: FrequencyRules<T>,
    adjacency_rules: AdjacencyRules<T>,
    /// TileId key
    inner: HashMap<u64, TileBordersAdjacency<T>>,
    /// BorderId key; (TileId; GridDir)
    border_types: HashMap<u64, Vec<(u64, GridDir)>>,
}

impl<T> Default for MSAnalyzer<T>
where
    T: IdentifiableTile,
{
    fn default() -> Self {
        Self {
            tiles: HashSet::new(),
            frequency_rules: FrequencyRules::default(),
            adjacency_rules: AdjacencyRules::default(),
            inner: HashMap::new(),
            border_types: HashMap::new(),
        }
    }
}

impl<T> AdjacencyAnalyzer<T> for MSAnalyzer<T>
where
    T: IdentifiableTile,
{
    fn analyze(&mut self, map: &GridMap2D<T>) {
      self.adjacency_rules = AdjacencyRules::default();
        for position in map.get_all_positions() {
            self.analyze_tile_at_pos(map, position);
        }
        self.generate_adjacency_rules();
        // self.inner.clear();
        // self.border_types.clear();
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

impl<T> MSAnalyzer<T>
where
    T: IdentifiableTile,
{
    fn analyze_tile_at_pos(&mut self, map: &GridMap2D<T>, pos: GridPos2D) {
        if let Some(tile) = map.get_tile_at_position(&pos) {
            self.tiles.insert(tile.get_tile_id());
            self.frequency_rules.count_tile(tile);

            for dir in GridDir::ALL {
                if let Some(neighbour) = map.get_neighbour_at(&pos, dir) {
                    self.add_adjacency_raw(tile.get_tile_id(), neighbour.get_tile_id(), dir);
                }
            }
        }
    }

    fn generate_adjacency_rules(&mut self) {
      let border_ids = self.border_types.keys().copied().collect::<Vec<_>>();

      for border_id in border_ids.iter() {
          let borders = self.border_types.get(border_id).unwrap().clone();
          let left_borders = borders
              .iter()
              .filter_map(|(tile, dir)| {
                  if *dir == GridDir::LEFT {
                      Some(*tile)
                  } else {
                      None
                  }
              })
              .collect::<Vec<_>>();
          let right_borders = borders
              .iter()
              .filter_map(|(tile, dir)| {
                  if *dir == GridDir::RIGHT {
                      Some(*tile)
                  } else {
                      None
                  }
              })
              .collect::<Vec<_>>();

          for tile_first in left_borders.iter() {
              for tile_second in right_borders.iter() {
                  self.adjacency_rules.add_adjacency_raw(*tile_first, *tile_second, GridDir::LEFT);
                  self.adjacency_rules.add_adjacency_raw(*tile_second, *tile_first, GridDir::RIGHT);
              }
          }

          let up_borders = borders
              .iter()
              .filter_map(|(tile, dir)| {
                  if *dir == GridDir::UP {
                      Some(*tile)
                  } else {
                      None
                  }
              })
              .collect::<Vec<_>>();
          let down_borders = borders
              .iter()
              .filter_map(|(tile, dir)| {
                  if *dir == GridDir::DOWN {
                      Some(*tile)
                  } else {
                      None
                  }
              })
              .collect::<Vec<_>>();

          for tile_first in up_borders.iter() {
              for tile_second in down_borders.iter() {
                  self.adjacency_rules.add_adjacency_raw(*tile_first, *tile_second, GridDir::UP);
                  self.adjacency_rules.add_adjacency_raw(*tile_second, *tile_first, GridDir::DOWN);
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
                self.set_border_id(id_border, tile_id, &direction.opposite());
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

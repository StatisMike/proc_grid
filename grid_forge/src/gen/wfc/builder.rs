use std::collections::BTreeMap;

use crate::{tile::GridTile2D, GridPos2D};

use super::{analyzer::WFCAnalyzer, WFCTile};

#[derive(Debug, Clone)]
pub struct WFCCloneBuilder<T>
where T: WFCTile + Clone
{
  tiles: BTreeMap<u64, T>
}

impl<T> Default for WFCCloneBuilder<T>
where T: WFCTile + Clone
{
    fn default() -> Self {
        Self { tiles: BTreeMap::new() }
    }
}

impl<T> WFCCloneBuilder<T> 
where T: WFCTile + Clone
{
  pub fn add_tiles(&mut self, tiles: Vec<&T>, overwrite: bool) {

    for tile in tiles {
      if !overwrite && self.tiles.contains_key(&tile.wfc_id()) {
        continue;
      }
      self.tiles.insert(tile.wfc_id(), tile.clone());
    }
  }
}

impl<T> WFCTileBuilder<T> for WFCCloneBuilder<T>
where T: WFCTile + Clone
{
    fn prepare_tile(&self, pos: GridPos2D, wfc_id: u64) -> T {
        let mut tile = self.tiles.get(&wfc_id).unwrap_or_else(|| panic!("can't get tile with {wfc_id}")).clone();
        tile.set_grid_position(pos);
        tile
    }
    
    fn missing_tiles(&self, analyzer: &WFCAnalyzer<T>) -> Vec<u64> {
        analyzer.tiles().iter().filter(|wfc_id| !self.tiles.contains_key(wfc_id)).copied().collect::<Vec<u64>>()
    }
}

#[derive(Debug, Clone)]
pub struct WFCFunBuilder<T>
where T: WFCTile
{
  funs: BTreeMap<u64, fn(GridPos2D, u64) -> T>
}

impl<T> Default for WFCFunBuilder<T>
where T: WFCTile
{
  fn default() -> Self {
      Self { funs: BTreeMap::new() }
  }
}

impl<T> WFCTileBuilder<T> for WFCFunBuilder<T>
where T: WFCTile
{
    fn prepare_tile(&self, pos: GridPos2D, wfc_id: u64) -> T {
        let fun = self.funs.get(&wfc_id).unwrap_or_else(|| panic!("can't get tile function with `wfc_id`: {wfc_id}"));

        fun(pos, wfc_id)
    }

    fn missing_tiles(&self, analyzer: &WFCAnalyzer<T>) -> Vec<u64> {
      analyzer.tiles().iter().filter(|wfc_id| !self.funs.contains_key(wfc_id)).copied().collect::<Vec<u64>>()
    }
}

pub trait WFCTileBuilder<T>
where T: GridTile2D + WFCTile {
  fn prepare_tile(&self, pos: GridPos2D, wfc_id: u64) -> T;

  fn missing_tiles(&self, analyzer: &WFCAnalyzer<T>) -> Vec<u64>;
}



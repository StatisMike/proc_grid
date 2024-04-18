use std::{borrow::BorrowMut, cell::RefCell, collections::VecDeque};

use crate::{map::{GridDir, GridMap2D, GridSize}, tile::GridTile2D, GridPos2D};

use super::{analyzer::{WFCAnalyzer, WFCTileProbs}, WFCTile};

use rand::{distributions::{Distribution, Uniform}, Rng};

#[derive(Clone)]
pub(crate) struct WFCGenTile {
  pos: GridPos2D,
  wfc_id: u64,
  options: Vec<u64>,
  collapsed: bool,
  entrophy: f32
}

impl GridTile2D for WFCGenTile {
  fn grid_position(&self) -> GridPos2D {
      self.pos
  }

  fn set_grid_position(&mut self, position: GridPos2D) {
      self.pos = position;
  }
}

impl WFCGenTile {

  pub fn new(pos: GridPos2D, options: Vec<u64>) -> Self {
    Self {
      pos,
      wfc_id: 0,
      options,
      collapsed: false,
      entrophy: 0.
    }
  }

  pub fn add_option(&mut self, wfc_id: u64) {
    if !self.options.contains(&wfc_id) {
      self.options.push(wfc_id)
    }
  }

  pub fn remove_option(&mut self, wfc_id: u64) {
    if let Some(idx) = self.options.iter().position(|x| x == &wfc_id) {
      self.options.remove(idx);
    }
  }

  pub fn is_collapsed(&self) -> bool {
    self.collapsed
  }

  pub fn collapse(&mut self, option: u64) {
    self.wfc_id = option;
    self.options.clear();
    self.collapsed = true;
  }

  pub fn collapse_last(&mut self) -> bool {
    if let Some(option) = self.options.last() {
      self.wfc_id = *option;
      self.options.clear();
      self.collapsed = true;
      return true;
    }
    false
  }

  pub fn try_collapse(&mut self) -> bool {
    if self.options.len() == 1 {
      self.wfc_id = self.options[0];
      self.options.clear();
      self.collapsed = true;
      true
    } else {
      false
    }
  }
}

pub struct WFCResolver<T>
where T: WFCTile
{
  wfc_grid: GridMap2D<WFCGenTile>,
  analyzer: WFCAnalyzer<T>,
  probs: WFCTileProbs,
  all_positions: Vec<GridPos2D>,
  dist: Uniform<f32>,
  changed: RefCell<VecDeque<GridPos2D>>
}

impl<T> WFCResolver<T>
where T: WFCTile
{
  pub fn new(size: GridSize, analyzer: WFCAnalyzer<T>) -> Self
  {
    let mut wfc_grid = GridMap2D::new(size);
    let probs = analyzer.probs();

    let options = probs.tiles_by_prob();

    let dist = Uniform::new(0., 1.);

    let all_positions = size.get_all_possible_positions();
    for position in all_positions.iter() {
      wfc_grid.insert_tile(WFCGenTile::new(*position, options.clone()));
    }

    let mut instance = Self {
      wfc_grid,
      analyzer,
      probs,
      all_positions: all_positions.clone(),
      dist,
      changed: RefCell::new(VecDeque::new())
    };

    for position in all_positions.iter() {
      instance.update_entrophy_at_pos(*position);
    }
    instance
  }

  pub fn process<R: Rng>(&mut self, rng: &mut R) -> bool {

    if let Some(pos) = self.select_lowest_entrophy_pos() {
      self.collapse_tile_at_pos(pos, rng).unwrap_or_else(|_| panic!("couln't collapse tile at {pos:?}"));
      self.resolve_tile_options_at_pos(pos);

      let mut changed = self.changed.borrow_mut().pop_front();

      while let Some(pos_changed) = changed {
        self.resolve_tile_options_at_pos(pos_changed);
        changed = self.changed.borrow_mut().pop_front();
      }
    }

    false
  }

  fn resolve_tile_options_at_pos(&mut self, pos: GridPos2D) {

    let tile = self.wfc_grid.get_tile_at_position(&pos).unwrap();
    let mut to_remove = Vec::new();

    // retrieve options to remove.
    for dir in GridDir::ALL {
      if let Some(neighbour) = self.wfc_grid.get_neighbour_at(&tile.grid_position(), dir) {
        
        for neighbour_option in neighbour.options.iter() {
          if !self.analyzer.is_valid_at_dir(*neighbour_option, tile.wfc_id, &dir.opposite()) {
            to_remove.push((neighbour.grid_position(), *neighbour_option));
          }
        }
      }
    }

    // remove unfitting options and update entrophy.
    while let Some((pos, option_id)) = to_remove.pop() {
      self.changed.borrow_mut().push_back(pos);
      self.wfc_grid.get_mut_tile_at_position(&pos).unwrap().remove_option(option_id);
      self.update_entrophy_at_pos(pos);
    }
  }

  fn update_entrophy_at_pos(&mut self, pos: GridPos2D) {
    let tile = self.wfc_grid.get_mut_tile_at_position(&pos).unwrap();

    if tile.collapsed {
      return;
    }

    tile.entrophy = self.probs.total_entropy(&tile.options);
  }

  fn get_entrophy_at_pos(&self, pos: GridPos2D) -> Option<f32>
  {
    let tile = self.wfc_grid.get_tile_at_position(&pos).unwrap();
    if tile.collapsed {
      return None;
    }
    Some(tile.entrophy)
  }

  fn collapse_tile_at_pos<R: Rng>(&mut self, pos: GridPos2D, rng: &mut R) -> Result<(), ()> {

    let tile = self.wfc_grid.get_mut_tile_at_position(&pos).unwrap();

    if tile.try_collapse() {
      self.changed.borrow_mut().push_back(pos);
      return Ok(());
    }

    let random = self.dist.sample(rng);

    for option in tile.options.iter() {
      if self.probs.wfc_prob(*option).unwrap() > random {
        tile.collapse(*option);
        self.changed.borrow_mut().push_back(pos);
        return Ok(())
      }
    }

    Err(())
  }

  fn select_lowest_entrophy_pos(&self) -> Option<GridPos2D>
  {
    let mut entrophies = self
      .all_positions
      .iter()
      .filter_map(|pos| self.get_entrophy_at_pos(*pos).map(|entrophy| (pos, entrophy)))
      .collect::<Vec<_>>();

    entrophies.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    entrophies.first().map(|e| e.0).copied()

  }
}
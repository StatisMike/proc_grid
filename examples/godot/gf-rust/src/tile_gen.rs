use godot::{builtin::{Array, GString}, engine::{AcceptDialog, ConfirmationDialog, Label, TileMap}, log::godot_warn, obj::Gd, register::{godot_api, GodotClass}};
use grid_forge::{gen::collapse::{AdjacencyAnalyzer, AdjacencyBorderAnalyzer, AdjacencyIdentityAnalyzer, AdjacencyRules, CollapsibleResolver, EntrophyQueue, FrequencyHints, PositionQueue}, map::{GridMap2D, GridSize}, tile::identifiable::{builders::IdentTileTraitBuilder, BasicIdentifiableTile2D}};
use rand::thread_rng;

use crate::tile_collections::TileCollections;

#[derive(GodotClass)]
#[class(base=Node, init)]
pub struct TileGenerator {
  #[export]
  collection: Option<Gd<TileCollections>>,
  #[export]
  modal: Option<Gd<AcceptDialog>>,
  #[export]
  runtime_error: Option<Gd<Label>>,
  #[var]
  running: bool,

  generated: Option<GridMap2D<BasicIdentifiableTile2D>>,

  border_rules: AdjacencyRules<BasicIdentifiableTile2D>,
  identity_rules: AdjacencyRules<BasicIdentifiableTile2D>,
  frequency_hints: FrequencyHints<BasicIdentifiableTile2D>,
}

#[godot_api]
impl TileGenerator {

  #[signal]
  fn generation_finished(success: bool);

  #[signal]
  fn generation_error(message: GString);

  #[signal]
  fn generation_runtime_error(message: GString);

  #[func]
  fn initialize_rulesets(&mut self, maps: Array<GString>) {
    let mut grid_maps = Vec::new();
    for map_path in maps.iter_shared() {
      match self.collection.as_ref().unwrap().bind().load_vis_map_from_path(&map_path.to_string()) {
          Ok(map) => {
            grid_maps.push(map);
          },
          Err(err) => {
            self.show_modal(&format!("Error loading map for TileGenerator: {err}"));
            return;
          },
      }
    }

    let mut analyzer = AdjacencyBorderAnalyzer::default();
      for map in grid_maps.iter() {
        analyzer.analyze(map);
      }
    self.border_rules = analyzer.adjacency().clone();

    let mut analyzer = AdjacencyIdentityAnalyzer::default();
      for map in grid_maps.iter() {
        analyzer.analyze(map)
      }
    self.identity_rules = analyzer.adjacency().clone();

    let mut frequency_hints = FrequencyHints::default();

    for map in grid_maps.iter() {
      frequency_hints.analyze_grid_map(map);
    }

    self.frequency_hints = frequency_hints;
  }

  fn on_generate_signal(&self, width: i32, height: i32, rule: i32, queue: i32, tilemap: Gd<TileMap>) {

  }

  #[func]
  fn generate(&self, width: i32, height: i32, rule: i32, queue: i32, tilemap: Gd<TileMap>) -> bool {

    let mut tilemap = tilemap.clone();
    let size = GridSize::new(width as u32, height as u32);
    let builder = IdentTileTraitBuilder::<BasicIdentifiableTile2D>::default();

    let adjacency_rules = if rule == 0 {
      self.border_rules.clone()
    } else {
      self.identity_rules.clone()
    };

    let mut resolver = CollapsibleResolver::new(size);
    let mut rng = thread_rng();

    let mut iter = 0;

    loop {
      let result = if queue == 0 {
        resolver.generate(
          &mut rng, 
          &size.get_all_possible_positions(), 
          PositionQueue::default(), 
          &self.frequency_hints, 
          &adjacency_rules
        )
      } else {
        resolver.generate(
          &mut rng, 
          &size.get_all_possible_positions(), 
          EntrophyQueue::default(), 
          &self.frequency_hints, 
          &adjacency_rules
        )
      };
      match result {
        Ok(_) => break,
        Err(err) => {
          if iter > 2 {
            self.show_modal(&format!("Cannot generate after 2 tries: {err}"));
            return false;
          }
          self.show_error(&format!("Error in iter: {iter}: {err}"));
          iter += 1;
        },
      }
    }
    
    let map = resolver.build_grid(&builder).unwrap();
    self.collection.as_ref().unwrap().bind().grid_map_into_tilemap(&map, &mut tilemap).unwrap();
    true
  }

  fn show_modal(&self, message: &str) {
    godot_warn!("Cannot find modal for TileGenerator. Message to show: {message}");
    if let Some(modal) = &self.modal {
      let mut pntr = modal.clone();
      pntr.set_text(message.into());
      pntr.set_visible(true);
    } else {
      godot_warn!("Cannot find modal for TileGenerator. Message to show: {message}");
    }
  }

  fn show_error(&self, message: &str) {
    godot_warn!("Cannot find runtime error label for TileGenerator. Message to show: {message}");
    if let Some(modal) = &self.runtime_error {
      let mut pntr = modal.clone();
      pntr.set_text(message.into());
    } else {
      godot_warn!("Cannot find runtime error label for TileGenerator. Message to show: {message}");
    }
  }
}
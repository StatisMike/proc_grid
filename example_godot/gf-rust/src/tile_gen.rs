//! Implements `grid_forge` collapse procedural generation algorithm, allowing its usage within Godot example app.

use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use godot::builtin::meta::{FromGodot, ToGodot};
use godot::builtin::{Array, Dictionary, GString, Vector2i};
use godot::engine::{AcceptDialog, INode, Node, TileMap, Timer};
use godot::log::{godot_error, godot_warn};
use godot::obj::{Base, Gd, NewAlloc, WithBaseField};
use godot::register::{godot_api, GodotClass};
use grid_forge::tile::{GridPosition, GridTile};
use singular::Analyzer;

use grid_forge::gen::collapse::*;
use grid_forge::map::*;
use grid_forge::tile::identifiable::builders::IdentTileTraitBuilder;
use grid_forge::tile::identifiable::BasicIdentTileData;

use rand::thread_rng;

use crate::tile_collections::{SingleTile, TileCollections};

#[derive(GodotClass)]
#[class(base=Node, init)]
pub struct TileGenerator {
    #[export]
    collection: Option<Gd<TileCollections>>,
    #[export]
    modal: Option<Gd<AcceptDialog>>,
    #[var]
    running: bool,
    #[var]
    pregenerated: Dictionary,

    handle: Option<JoinHandle<()>>,
    channel: Option<Receiver<GenerationResult>>,
    generated: Option<GridMap2D<BasicIdentTileData>>,

    #[var]
    generation_time_us_total: u32,
    #[var]
    generation_time_us_success: u32,
    generation_history: Vec<singular::CollapseHistoryItem>,

    border_rules: singular::AdjacencyRules<BasicIdentTileData>,
    identity_rules: singular::AdjacencyRules<BasicIdentTileData>,
    frequency_hints: singular::FrequencyHints<BasicIdentTileData>,

    base: Base<Node>,
}

#[godot_api]
impl INode for TileGenerator {
    fn process(&mut self, _delta: f64) {
        if !self.running {
            return;
        }

        // Check if there is a result available from the generation task from separate thread. If so, emit the signal
        // to Godot's *MainNode*.
        if let Some(receiver) = &self.channel {
            if let Ok(result) = receiver.try_recv() {
                match result {
                    GenerationResult::RuntimeErr(mssg) => {
                        self.base_mut().emit_signal(
                            "generation_runtime_error".into(),
                            &[GString::from(mssg).to_variant()],
                        );
                    }
                    GenerationResult::Error(mssg) => {
                        self.base_mut().emit_signal(
                            "generation_error".into(),
                            &[GString::from(mssg).to_variant()],
                        );
                        self.base_mut()
                            .emit_signal("generation_finished".into(), &[false.to_variant()]);
                        if self.handle.is_some() {
                            let thread = self.handle.take();
                            thread.unwrap().join().unwrap();
                        }
                        self.running = false;
                    }
                    GenerationResult::Success((
                        map,
                        history,
                        (duration_total, duration_success),
                    )) => {
                        (self.generated, self.generation_history) = (Some(map), history);
                        self.generation_time_us_total = duration_total.as_micros() as u32;
                        self.generation_time_us_success = duration_success.as_micros() as u32;
                        self.base_mut()
                            .emit_signal("generation_finished".into(), &[true.to_variant()]);
                        if self.handle.is_some() {
                            let thread = self.handle.take();
                            thread.unwrap().join().unwrap();
                        }
                        self.running = false;
                    }
                }
            }
        }
    }
}

#[godot_api]
impl TileGenerator {
    /// Emitted when the generation is finished.
    #[signal]
    fn generation_finished(success: bool);

    /// Emitted when the generation encounters an error and will be stopped.
    #[signal]
    fn generation_error(message: GString);

    /// Emitted if the generation encounters an error, but will retry.
    #[signal]
    fn generation_runtime_error(message: GString);

    /// Initializes the single-tiled rulesets for the generation.
    #[func]
    fn initialize_rulesets(&mut self, maps: Array<GString>) {
        let mut grid_maps = Vec::new();
        for map_path in maps.iter_shared() {
            match self
                .collection
                .as_ref()
                .unwrap()
                .bind()
                .load_vis_map_from_path(&map_path.to_string())
            {
                Ok(map) => {
                    grid_maps.push(map);
                }
                Err(err) => {
                    self.show_modal(&format!("Error loading map for TileGenerator: {err}"));
                    return;
                }
            }
        }

        let mut analyzer = singular::BorderAnalyzer::default();
        for map in grid_maps.iter() {
            analyzer.analyze(map);
        }
        self.border_rules = analyzer.adjacency().clone();

        let mut analyzer = singular::IdentityAnalyzer::default();
        for map in grid_maps.iter() {
            analyzer.analyze(map)
        }
        self.identity_rules = analyzer.adjacency().clone();

        let mut frequency_hints = singular::FrequencyHints::default();

        for map in grid_maps.iter() {
            frequency_hints.analyze_grid_map(map);
        }

        self.frequency_hints = frequency_hints;
    }

    /// Starts the generation of the tilemap. Whole generation will be done in a separate thread, so the main thread will not be blocked.
    #[func]
    fn begin_generation(
        &mut self,
        width: i32,
        height: i32,
        rule: i32,
        queue: i32,
        pregenerated: Dictionary,
    ) {
        let (sender, receiver) = mpsc::channel();
        let subscriber = singular::CollapseHistorySubscriber::default();
        let mut collapsed_grid = None;
        self.channel = Some(receiver);
        self.running = true;
        let size = GridSize::new_xy(width as u32, height as u32);

        let collapsed = self.unpack_pregenerated(&pregenerated);

        if !collapsed.is_empty() {
            let mut grid = CollapsedGrid::new(size);
            for (pos, single_tile) in collapsed.iter() {
                grid.insert_tile(GridTile::new(
                    *pos,
                    CollapsedTileData::new(single_tile.bind().get_tile_type_id() as u64),
                ));
            }
            collapsed_grid = Some(grid);
        }
        self.pregenerated = pregenerated;

        let adjacency_rules = if rule == 0 {
            self.border_rules.clone()
        } else {
            self.identity_rules.clone()
        };

        let frequency_hints = self.frequency_hints.clone();

        let builder = IdentTileTraitBuilder::<BasicIdentTileData>::default();

        self.handle = Some(thread::spawn(move || {
            const RETRY_COUNT: usize = 10;

            let mut iter = 0;
            let mut rng = thread_rng();
            let mut resolver = singular::Resolver::default().with_subscriber(Box::new(subscriber));

            let mut grid =
                singular::CollapsibleTileGrid::new_empty(size, &frequency_hints, &adjacency_rules);

            if let Some(collapsed) = collapsed_grid {
                if let Err(err) = grid.populate_from_collapsed(&collapsed) {
                    sender
                        .send(GenerationResult::Error(format!(
                            "Cannot populate grid from collapsed grid: {err}"
                        )))
                        .unwrap();
                    return;
                }
            }

            let empty_positions = grid.empty_positions();

            let mut duration_total: Duration;
            let mut duration_success: Duration;

            let start_total = Instant::now();

            loop {
                let start_success = Instant::now();
                let result = if queue == 0 {
                    resolver.generate_position(
                        &mut grid,
                        &mut rng,
                        &empty_positions,
                        PositionQueue::default(),
                    )
                } else {
                    resolver.generate_entrophy(&mut grid, &mut rng, &empty_positions)
                };
                duration_total = start_total.elapsed();
                duration_success = start_success.elapsed();
                match result {
                    Ok(_) => break,
                    Err(err) => {
                        if iter >= RETRY_COUNT {
                            sender
                                .send(GenerationResult::Error(format!(
                                    "Cannot generate after {RETRY_COUNT} tries: {err}"
                                )))
                                .unwrap();
                            return;
                        }
                        sender
                            .send(GenerationResult::RuntimeErr(format!(
                                "Error in iter: {iter}: {err}"
                            )))
                            .unwrap();
                        iter += 1;
                    }
                }
            }

            let map = grid.retrieve_ident(&builder).unwrap();
            let subscriber = resolver.retrieve_subscriber().unwrap();
            let history = subscriber
                .as_any()
                .downcast_ref::<singular::CollapseHistorySubscriber>()
                .unwrap()
                .history()
                .to_vec();
            sender
                .send(GenerationResult::Success((
                    map,
                    history,
                    (duration_total, duration_success),
                )))
                .unwrap();
        }));
    }

    /// Transfers the generated [`GridMap2D`] to the Godot's [`TileMap`].
    #[func]
    fn generated_to_tilemap(&self, tilemap: Gd<TileMap>) -> bool {
        let mut tilemap = tilemap.clone();
        if let Some(map) = &self.generated {
            self.collection
                .as_ref()
                .unwrap()
                .bind()
                .grid_map_into_tilemap(map, &mut tilemap)
                .unwrap();
            return true;
        }
        false
    }

    fn show_modal(&self, message: &str) {
        if let Some(modal) = &self.modal {
            let mut pntr = modal.clone();
            pntr.set_text(message.into());
            pntr.set_visible(true);
        } else {
            godot_warn!("Cannot find modal for TileGenerator. Message to show: {message}");
        }
    }

    fn get_generation_result(
        &self,
    ) -> Option<(
        &GridMap2D<BasicIdentTileData>,
        &Vec<singular::CollapseHistoryItem>,
    )> {
        if let Some(map) = &self.generated {
            return Some((map, &self.generation_history));
        }
        None
    }

    fn unpack_pregenerated(
        &self,
        pregenerated: &Dictionary,
    ) -> Vec<(GridPosition, Gd<SingleTile>)> {
        let mut out = Vec::new();
        for (key, value) in pregenerated.iter_shared() {
            let pos = Vector2i::from_variant(&key);
            let tile = Gd::<SingleTile>::from_variant(&value);
            out.push((GridPosition::new_xy(pos.x as u32, pos.y as u32), tile));
        }
        out
    }
}

/// Result of the [`TileGenerator`] generation, received from the spawned thread, to be send into Godot's *MainNode*.
enum GenerationResult {
    /// Runtime error - only passing the error message to Godot, generator will retry.
    RuntimeErr(String),
    /// Fatal error, the generation will be stopped.
    Error(String),
    /// Successful generation - the generated map will be sent to Godot's *MainNode*.
    Success(
        (
            GridMap2D<BasicIdentTileData>,
            Vec<singular::CollapseHistoryItem>,
            (Duration, Duration),
        ),
    ),
}

#[derive(GodotClass)]
#[class(base=Node, init)]
pub struct GenerationHistoryState {
    playing: bool,
    timer: Option<Gd<Timer>>,
    #[export]
    tilemap: Option<Gd<TileMap>>,
    #[export]
    collection: Option<Gd<TileCollections>>,
    history: Vec<singular::CollapseHistoryItem>,
    #[var]
    current: u32,
    #[var]
    total: u32,
    base: Base<Node>,
}

impl GenerationHistoryState {
    fn add_from_current(&mut self) -> bool {
        if self.current >= self.total {
            return false;
        };
        let (Some(map), Some(collection)) = (&mut self.tilemap, &self.collection) else {
            godot_error!("Cannot draw frame, because either tilemap or collection is not set");
            return false;
        };
        let Some(item) = self.history.get(self.current as usize - 1) else {
            godot_error!("Cannot draw frame, because history is empty");
            return false;
        };
        collection.bind().insert_tile(
            map.clone(),
            item.tile_type_id,
            item.position.get_godot_coords(),
        );
        true
    }

    fn remove_from_current(&mut self) -> bool {
        if self.current == 0 {
            return false;
        }
        let Some(map) = &mut self.tilemap else {
            godot_error!("Cannot draw frame, because either tilemap or collection is not set");
            return false;
        };
        let Some(item) = self.history.get(self.current as usize - 1) else {
            godot_error!("Cannot draw frame, because history is empty");
            return false;
        };
        map.set_cell(0, item.position.get_godot_coords());
        true
    }
}

#[godot_api]
impl GenerationHistoryState {
    #[signal]
    fn current_state(current: u32);

    #[func]
    pub fn set_history_from_generator(&mut self, generator: Gd<TileGenerator>) {
        let binding = generator.bind();
        let Some((gridmap, history)) = binding.get_generation_result() else {
            godot_error!("Cannot get generation result from generator");
            return;
        };

        if let Some(map) = &mut self.tilemap {
            map.call(
                "adjust_generation".into(),
                &[
                    Vector2i::new(gridmap.size().x() as i32, gridmap.size().y() as i32)
                        .to_variant(),
                ],
            );
        }
        self.history.clone_from(history);
        self.current = 0;
        self.total = self.history.len() as u32 + 1;
        self.base_mut()
            .emit_signal("current_state".into(), &[0.to_variant()]);
    }

    #[func]
    fn play(&mut self, count_per_sec: u32) {
        if self.current >= self.total {
            return;
        }
        self.playing = true;
        let mut timer = Timer::new_alloc();
        self.base_mut().add_child(timer.clone().upcast());
        timer.set_wait_time(1. / count_per_sec as f64);
        timer.connect("timeout".into(), self.base_mut().callable("forward"));
        timer.start();
        self.timer = Some(timer);
    }

    #[func]
    fn stop(&mut self) {
        self.playing = false;
        if self.timer.is_some() {
            let mut timer = self.timer.take().unwrap();
            timer.stop();
            timer.queue_free();
        }
    }

    #[func]
    fn forward(&mut self) {
        if self.current >= self.total {
            return;
        }
        self.current += 1;
        if !self.add_from_current() && self.timer.is_some() {
            let mut timer = self.timer.take().unwrap();
            timer.stop();
            timer.queue_free();
            self.playing = false;
        }
        let current = self.current;
        self.base_mut()
            .emit_signal("current_state".into(), &[current.to_variant()]);
    }

    #[func]
    fn backward(&mut self) {
        self.remove_from_current();
        if self.current == 0 {
            return;
        }
        self.current -= 1;
        let current = self.current;
        self.base_mut()
            .emit_signal("current_state".into(), &[current.to_variant()]);
    }

    #[func]
    fn rewind(&mut self) {
        self.remove_from_current();
        while self.current > 0 {
            self.current -= 1;
            self.remove_from_current();
        }
    }
}

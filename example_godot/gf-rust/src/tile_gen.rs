//! Implements `grid_forge` collapse procedural generation algorithm, allowing its usage within Godot example app.

use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};

use godot::builtin::meta::ToGodot;
use godot::builtin::{Array, GString, Vector2i};
use godot::engine::{AcceptDialog, INode, Node, TileMap};
use godot::log::godot_warn;
use godot::obj::{Base, Gd, WithBaseField};
use godot::register::{godot_api, GodotClass};
use grid_forge::tile::GridPosition;
use singular::Analyzer;

use grid_forge::gen::collapse::*;
use grid_forge::map::*;
use grid_forge::tile::identifiable::builders::IdentTileTraitBuilder;
use grid_forge::tile::identifiable::BasicIdentTileData;

use rand::thread_rng;

use crate::tile_collections::TileCollections;

#[derive(GodotClass)]
#[class(base=Node, init)]
pub struct TileGenerator {
    #[export]
    collection: Option<Gd<TileCollections>>,
    #[export]
    modal: Option<Gd<AcceptDialog>>,
    #[var]
    running: bool,

    handle: Option<JoinHandle<()>>,
    channel: Option<Receiver<GenerationResult>>,
    generated: Option<GridMap2D<BasicIdentTileData>>,

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
                    GenerationResult::Success(map) => {
                        self.generated = Some(map);
                        self.base_mut()
                            .emit_signal("generation_finished".into(), &[true.to_variant()]);
                        if self.handle.is_some() {
                            let thread = self.handle.take();
                            thread.unwrap().join().unwrap();
                        }
                        self.running = false;
                    }
                    GenerationResult::CollapsedTile(position, tile_type_id) => {
                        self.base_mut().emit_signal(
                            "generation_collapsed".into(),
                            &[
                                position.get_godot_coords().to_variant(),
                                tile_type_id.to_variant(),
                            ],
                        );
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

    /// Emitted when the generation encounters an error an will be stopped.
    #[signal]
    fn generation_error(message: GString);

    /// Emitted if the generation encounters an error, but will retry.
    #[signal]
    fn generation_runtime_error(message: GString);

    /// Emitted when the singular tile have been collapsed during the generation, if the generation was subscribed to.
    #[signal]
    fn generation_collapsed(coords: Vector2i, tile_type_id: u64);

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
        subscribe: bool,
    ) {
        let (sender, receiver) = mpsc::channel();
        let mut subscriber = None;
        if subscribe {
            subscriber = Some(SenderSubscriber::new(sender.clone()));
        }
        self.channel = Some(receiver);
        self.running = true;

        let adjacency_rules = if rule == 0 {
            self.border_rules.clone()
        } else {
            self.identity_rules.clone()
        };

        let frequency_hints = self.frequency_hints.clone();

        let size = GridSize::new_xy(width as u32, height as u32);
        let builder = IdentTileTraitBuilder::<BasicIdentTileData>::default();

        self.handle = Some(thread::spawn(move || {
            const RETRY_COUNT: usize = 10;

            let mut iter = 0;
            let mut rng = thread_rng();
            let mut resolver = singular::Resolver::default();
            if let Some(subscriber) = subscriber {
                resolver = resolver.with_subscriber(Box::new(subscriber));
            }
            let mut grid =
                singular::CollapsibleTileGrid::new_empty(size, &frequency_hints, &adjacency_rules);

            loop {
                let result = if queue == 0 {
                    resolver.generate(
                        &mut grid,
                        &mut rng,
                        &size.get_all_possible_positions(),
                        PositionQueue::default(),
                    )
                } else {
                    resolver.generate(
                        &mut grid,
                        &mut rng,
                        &size.get_all_possible_positions(),
                        EntrophyQueue::default(),
                    )
                };
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
            sender.send(GenerationResult::Success(map)).unwrap();
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
}

/// Result of the [`TileGenerator`] generation, received from the spawned thread, to be send into Godot's *MainNode*.
enum GenerationResult {
    /// Runtime error - only passing the error message to Godot, generator will retry.
    RuntimeErr(String),
    /// Fatal error, the generation will be stopped.
    Error(String),
    /// Collapsed tile - the tile has been collapsed will be inserted into the tilemap. Contains the position and the `tile_type_id`
    /// of the collapsed tile. They will be passed to Godot's *MainNode*.
    CollapsedTile(GridPosition, u64),
    /// Successful generation - the generated map will be sent to Godot's *MainNode*.
    Success(GridMap2D<BasicIdentTileData>),
}

/// Resolver Subsciber sending the result of the generation through underlying [`Sender`](mpsc::Sender).
struct SenderSubscriber {
    sender: mpsc::Sender<GenerationResult>,
}

impl SenderSubscriber {
    pub fn new(sender: mpsc::Sender<GenerationResult>) -> Self {
        Self { sender }
    }
}

impl singular::Subscriber for SenderSubscriber {
    fn on_collapse(&mut self, position: &grid_forge::tile::GridPosition, tile_type_id: u64) {
        // Delay the sending to let the Godot react to the signal
        std::thread::sleep(std::time::Duration::from_millis(10));
        self.sender
            .send(GenerationResult::CollapsedTile(*position, tile_type_id))
            .unwrap();
    }
}

use std::{
    sync::mpsc::{self, Receiver},
    thread::{self, JoinHandle},
};

use godot::{
    builtin::{meta::ToGodot, Array, GString},
    engine::{AcceptDialog, INode, Node, TileMap},
    log::godot_warn,
    obj::{Base, Gd, WithBaseField},
    register::{godot_api, GodotClass},
};
use grid_forge::{
    gen::collapse::{
        AdjacencyAnalyzer, AdjacencyBorderAnalyzer, AdjacencyIdentityAnalyzer, AdjacencyRules,
        CollapsibleResolver, EntrophyQueue, FrequencyHints, PositionQueue,
    },
    map::{GridMap2D, GridSize},
    tile::identifiable::{builders::IdentTileTraitBuilder, BasicIdentifiableTile2D},
};
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
    generated: Option<GridMap2D<BasicIdentifiableTile2D>>,

    border_rules: AdjacencyRules<BasicIdentifiableTile2D>,
    identity_rules: AdjacencyRules<BasicIdentifiableTile2D>,
    frequency_hints: FrequencyHints<BasicIdentifiableTile2D>,

    base: Base<Node>,
}

#[godot_api]
impl INode for TileGenerator {
    fn process(&mut self, _delta: f64) {
        if !self.running {
            return;
        }

        // Check if there is a result available from the generation task
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
                }
            }
        }
    }
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

    #[func]
    fn begin_generation(&mut self, width: i32, height: i32, rule: i32, queue: i32) {
        let (sender, receiver) = mpsc::channel();
        self.channel = Some(receiver);
        self.running = true;

        let adjacency_rules = if rule == 0 {
            self.border_rules.clone()
        } else {
            self.identity_rules.clone()
        };

        let frequency_hints = self.frequency_hints.clone();

        let size = GridSize::new(width as u32, height as u32);
        let builder = IdentTileTraitBuilder::<BasicIdentifiableTile2D>::default();

        self.handle = Some(thread::spawn(move || {
            const RETRY_COUNT: usize = 10;

            let mut iter = 0;
            let mut rng = thread_rng();
            let mut resolver = CollapsibleResolver::new(size);

            loop {
                let result = if queue == 0 {
                    resolver.generate(
                        &mut rng,
                        &size.get_all_possible_positions(),
                        PositionQueue::default(),
                        &frequency_hints,
                        &adjacency_rules,
                    )
                } else {
                    resolver.generate(
                        &mut rng,
                        &size.get_all_possible_positions(),
                        EntrophyQueue::default(),
                        &frequency_hints,
                        &adjacency_rules,
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

            let map = resolver.build_grid(&builder).unwrap();
            sender.send(GenerationResult::Success(map)).unwrap();
        }));
    }

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

enum GenerationResult {
    RuntimeErr(String),
    Error(String),
    Success(GridMap2D<BasicIdentifiableTile2D>),
}

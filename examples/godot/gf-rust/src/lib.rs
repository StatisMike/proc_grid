//! Example shows `godot` feature of `grid_forge` when working with single-layered [`TileMap`](godot::engine::TileMap).

use godot::init::*;

pub mod tests;
pub mod tile_collections;
pub mod tile_gen;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

use godot::{engine::TileSet, prelude::*};
use grid_forge::{
    tile::{identifiable::IdentifiableTile, GridTile2D},
    GridPos2D,
};

pub mod tests;
pub mod tile_collections;
pub mod tile_gen;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

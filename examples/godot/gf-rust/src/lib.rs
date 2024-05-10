use godot::{engine::TileSet, prelude::*};
use grid_forge::{
    tile::{identifiable::IdentifiableTile, GridTile2D},
    GridPos2D,
};

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

#[derive(Clone, Copy)]
struct GodotTile {
    pos: GridPos2D,
    tile_id: u64,
}

impl GridTile2D for GodotTile {
    fn grid_position(&self) -> GridPos2D {
        self.pos
    }
    fn set_grid_position(&mut self, position: GridPos2D) {
        self.pos = position;
    }
}

impl IdentifiableTile for GodotTile {
    fn tile_type_id(&self) -> u64 {
        self.tile_id
    }
}

fn from_tileset(path: impl Into<GString>) {
    let tileset = load::<TileSet>(path);
    // tileset.tiles
}

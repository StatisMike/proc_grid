use godot::init::*;

pub mod tests;
pub mod tile_collections;
pub mod tile_gen;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

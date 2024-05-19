use godot::builtin::Vector2i;

use crate::tile::GridPosition;

pub mod collection;
pub mod error;
pub mod ops;

#[derive(Clone, Copy)]
pub(crate) enum TileSourceType {
    Atlas,
    Collection,
    Mesh,
}

impl GridPosition {
    pub fn from_godot_v2i(coords: Vector2i) -> Self {
        Self::new_xy(coords.x as u32, coords.y as u32)
    }

    pub fn from_godot_v2i_layer(coords: Vector2i, layer: i32) -> Self {
        Self::new_xyz(coords.x as u32, coords.y as u32, layer as u32)
    }

    pub fn get_godot_coords(&self) -> Vector2i {
        Vector2i {
            x: *self.x() as i32,
            y: *self.y() as i32,
        }
    }

    pub fn get_godot_layer(&self) -> Option<i32> {
        self.z().map(|layer| layer as i32)
    }
}

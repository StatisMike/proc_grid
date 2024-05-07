use crate::GridPos2D;

pub mod identifiable;

#[cfg(feature = "vis")]
pub mod vis;

/// Trait that needs to be implemented for objects contained within the [`GridMap2D`](crate::map::GridMap2D).
pub trait GridTile2D {
    fn grid_position(&self) -> GridPos2D;

    fn set_grid_position(&mut self, position: GridPos2D);
}

// TODO! Remodel GridPosition from simple type to struct, allowing for layered gridmaps and different grids than rectangular.
// pub trait GridPosition {
//     /// Retrieves horizontal position on two dimensional grid.
//     fn x() -> u32;
//     /// Retrieves vertical position on two dimensional grid.
//     fn y() -> u32;
//     /// Retrieves layer number if position is part of layered grid, or `None` if it is not.
//     fn layer() -> Option<u32>;
// }

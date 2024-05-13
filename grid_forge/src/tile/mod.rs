use crate::GridPos2D;

pub mod identifiable;

#[cfg(feature = "vis")]
pub mod vis;

/// Trait that needs to be implemented for objects contained within the [`GridMap2D`](crate::map::GridMap2D).
pub trait GridTile2D {
    fn grid_position(&self) -> GridPos2D;

    fn set_grid_position(&mut self, position: GridPos2D);
}

#[derive(Debug)]
pub struct GridTile<Data>
{
    position: GridPosition,
    data: Data
}

impl<Data> GridTile<Data>
{
    pub fn new(position: GridPosition, data: Data) -> Self
    {
        Self { position, data }
    }

    pub fn grid_position(&self) -> &GridPosition
    {
        &self.position
    }

    pub fn inner(&self) -> &Data
    {
        &self.data
    }

    pub fn inner_mut(&mut self) -> &mut Data
    {
        &mut self.data
    }
}

#[derive(Clone, Copy, Debug, Hash)]
pub struct GridPosition {
    x: u32,
    y: u32,
    z: Option<u32>
}

impl GridPosition {
    pub fn new_xy(x: u32, y: u32) -> Self {
        Self { x, y, z: None }
    }

    pub fn new_xyz(x: u32, y: u32, z: u32) -> Self {
        Self { x, y, z: Some(z) }
    }

    pub fn x(&self) -> &u32 {
        &self.x
    }
    pub fn y(&self) -> &u32 {
        &self.y
    }
    pub fn z(&self) -> &Option<u32> {
        &self.z
    }
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

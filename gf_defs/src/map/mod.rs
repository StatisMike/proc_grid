use std::collections::HashMap;

use crate::tile::GridTile2D;
use crate::{add_grid_positions, GridPos2D};

use size::GridSize;

pub mod size;

#[cfg(feature = "image")]
pub mod vis;

#[repr(u8)]
#[derive(PartialEq, Eq, Debug)]
pub enum GridDir {
    UP = 1,
    DOWN = 2,
    LEFT = 3,
    RIGHT = 4,
}

impl GridDir {
    pub const ALL: &[GridDir; 4] = &[GridDir::UP, GridDir::DOWN, GridDir::LEFT, GridDir::RIGHT];

    pub fn march_step(&self, from: &GridPos2D, size: &GridSize) -> Option<GridPos2D> {
        let (x_dif, y_dif) = match self {
            GridDir::UP => {
                if from.1 == 0 {
                    return None;
                }
                (0i32, -1i32)
            }
            GridDir::DOWN => {
                if from.1 + 1 == size.y() {
                    return None;
                }
                (0i32, 1i32)
            }
            GridDir::LEFT => {
                if from.0 == 0 {
                    return None;
                }
                (-1i32, 0i32)
            }
            GridDir::RIGHT => {
                if from.0 + 1 == size.x() {
                    return None;
                }
                (1i32, 0i32)
            }
        };
        Some((
            (x_dif.wrapping_add_unsigned(from.0)) as u32,
            (y_dif.wrapping_add_unsigned(from.1)) as u32,
        ))
    }
}

/// Basic two-dimensional GridMap.
///
/// Grid Map holds data of all objects inside, with their [GridPosition2D] and [GridLayer] for easy access and additional
/// methods for handling them.
///
/// Extend of created GridMap usage stems from additional traits that are implemented for collected objects, with
/// [GridTile2D] at minimum.
pub struct GridMap2D<T>
where
    T: GridTile2D,
{
    pub(crate) size: GridSize,
    tiles: HashMap<GridPos2D, T>,
}

impl<T: GridTile2D> GridMap2D<T> {
    /// Creates new, empty map of given size.
    pub fn new(size: GridSize) -> Self {
        Self {
            size,
            tiles: HashMap::new(),
        }
    }

    /// Get tile at specified position.
    pub fn get_tile_at_position(&self, position: &GridPos2D) -> Option<&T> {
        if !self.size.is_position_valid(position) {
            return None;
        }
        self.tiles.get(position)
    }

    /// Get tile at specified position mutably.
    pub fn get_mut_tile_at_position(&mut self, position: &GridPos2D) -> Option<&mut T> {
        if !self.size.is_position_valid(position) {
            return None;
        }
        self.tiles.get_mut(position)
    }

    /// Insert tile. Its position will be determined based on information in [GridTile2D::grid_position]. If tile is
    /// present at that position already, it will be overwritten.
    pub fn insert_tile(&mut self, tile: T) -> bool {
        if !self.size.is_position_valid(&tile.grid_position()) {
            return false;
        }
        self.tiles.insert(tile.grid_position(), tile);
        true
    }

    pub fn size(&self) -> &GridSize {
        &self.size
    }

    /// Get tiles neighbouring the specified position.
    pub fn get_neighbours(&self, position: &GridPos2D) -> Vec<&T> {
        GridDir::ALL
            .iter()
            .filter_map(|direction| self.get_neighbour_at(position, direction))
            .collect::<Vec<_>>()
    }

    /// Get tile neighbouring the specified position at specified direction.
    pub fn get_neighbour_at(&self, position: &GridPos2D, direction: &GridDir) -> Option<&T> {
        if let Some(position) = direction.march_step(position, &self.size) {
            return self.get_tile_at_position(&position);
        }
        None
    }

    /// Get positions of all tiles within the GridMap
    pub fn get_all_positions(&self) -> Vec<GridPos2D> {
        self.tiles.keys().copied().collect::<Vec<GridPos2D>>()
    }

    /// Get positions of all tiles that are in the border
    pub fn get_all_border_positions(&self, direction: &GridDir) -> Vec<GridPos2D> {
        self.tiles
            .keys()
            .filter(|position| self.get_neighbour_at(position, direction).is_none())
            .copied()
            .collect::<Vec<GridPos2D>>()
    }

    pub fn get_all_empty_positions(&self) -> Vec<GridPos2D> {
        let mut out = Vec::new();

        for possible in self.size.get_all_possible_positions() {
            if !self.tiles.contains_key(&possible) {
                out.push(possible);
            }
        }

        out
    }

    /// Destroys the GridMap, returning all tiles with their position adjusted
    pub fn drain_remapped(mut self, anchor_pos: GridPos2D) -> Vec<T> {
        let mut out: Vec<T> = Vec::new();

        for (position, mut tile) in self.tiles.drain() {
            tile.set_grid_position(add_grid_positions(anchor_pos, position));
            out.push(tile);
        }

        out
    }
}

impl<T: GridTile2D + Default> GridMap2D<T> {
    pub fn fill_empty_with_default(&mut self) {
        for pos in self.get_all_empty_positions() {
            self.insert_tile({
                let mut tile = T::default();
                tile.set_grid_position(pos);
                tile
            });
        }
    }
}

impl<T: GridTile2D + Clone> GridMap2D<T> {
    pub fn fill_empty_with(&mut self, tile: T) {
        for pos in self.get_all_empty_positions() {
            self.insert_tile({
                let mut tile = tile.clone();
                tile.set_grid_position(pos);
                tile
            });
        }
    }

    /// Get all tiles with their positions remapped according to `anchor_pos`, which is the `left-top` position.
    pub fn get_remapped(&self, anchor_pos: GridPos2D) -> Vec<T> {
        let mut out: Vec<T> = Vec::new();

        for (pos, tile) in self.tiles.iter() {
            let mut cloned = tile.clone();
            cloned.set_grid_position(add_grid_positions(anchor_pos, *pos));
            out.push(cloned);
        }

        out
    }
}

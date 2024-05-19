use grid::Grid;

use crate::tile::{GridPosition, GridTile, GridTileRef, GridTileRefMut, TileContainer, TileData};

#[repr(u8)]
#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum GridDir {
    UP = 1,
    DOWN = 2,
    LEFT = 3,
    RIGHT = 4,
}

impl GridDir {
    /// All possible directions from tile to tile within a [GridMap2D].
    pub const ALL_2D: &'static [GridDir; 4] =
        &[GridDir::UP, GridDir::DOWN, GridDir::LEFT, GridDir::RIGHT];

    /// Take a step in specified direction from position within the contains of specified [GridSize].
    ///
    /// # Returns
    /// - resulting [GridPos2D] after the step, or [None] if position is not valid within the specified size.
    ///
    /// # Examples
    /// ```
    /// use grid_forge::tile::GridPosition;
    /// use grid_forge::map::GridDir;
    /// use grid_forge::map::GridSize;
    ///
    /// let size = GridSize::new_xy(3, 3);
    /// let position = GridPosition::new_xy(0, 1);
    ///
    /// assert_eq!(Some(GridPosition::new_xy(0, 0)), GridDir::UP.march_step(&position, &size));
    /// assert_eq!(None, GridDir::LEFT.march_step(&position, &size));
    /// ```
    pub fn march_step(&self, from: &GridPosition, size: &GridSize) -> Option<GridPosition> {
        let (x_dif, y_dif, z_dif) = match self {
            GridDir::UP => {
                if from.y() == &0 {
                    return None;
                }
                (0i32, -1i32, 0i32)
            }
            GridDir::DOWN => {
                if from.y() + 1 == size.y() {
                    return None;
                }
                (0i32, 1i32, 0i32)
            }
            GridDir::LEFT => {
                if from.x() == &0 {
                    return None;
                }
                (-1i32, 0i32, 0i32)
            }
            GridDir::RIGHT => {
                if from.x() + 1 == size.x() {
                    return None;
                }
                (1i32, 0i32, 0i32)
            }
        };
        let (x, y, z) = (
            (x_dif.wrapping_add_unsigned(*from.x())) as u32,
            (y_dif.wrapping_add_unsigned(*from.y())) as u32,
            from.z().map(|z| z_dif.wrapping_add_unsigned(z) as u32),
        );

        if let Some(z) = z {
            Some(GridPosition::new_xyz(x, y, z))
        } else {
            Some(GridPosition::new_xy(x, y))
        }
    }

    /// Get opposite direction.
    ///
    /// # Examples
    /// ```
    /// use grid_forge::map::GridDir;
    ///
    /// assert_eq!(GridDir::UP, GridDir::DOWN.opposite())
    /// ```
    #[inline]
    pub fn opposite(&self) -> Self {
        match self {
            GridDir::UP => GridDir::DOWN,
            GridDir::DOWN => GridDir::UP,
            GridDir::LEFT => GridDir::RIGHT,
            GridDir::RIGHT => GridDir::LEFT,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GridSize {
    x: u32,
    y: u32,
    z: Option<u32>,
    center: (u32, u32),
}

impl GridSize {
    pub fn new_xy(x: u32, y: u32) -> Self {
        let center = Self::calc_center_approx(x, y);
        Self {
            x,
            y,
            z: None,
            center,
        }
    }

    pub fn new_xyz(x: u32, y: u32, z: u32) -> Self {
        let center = Self::calc_center_approx(x, y);
        Self {
            x,
            y,
            z: Some(z),
            center,
        }
    }

    pub fn x(&self) -> u32 {
        self.x
    }

    pub fn y(&self) -> u32 {
        self.y
    }

    pub fn center(&self) -> (u32, u32) {
        self.center
    }

    pub fn is_position_valid(&self, position: &GridPosition) -> bool {
        position.x() < &self.x && position.y() < &self.y
    }

    pub fn get_all_possible_positions(&self) -> Vec<GridPosition> {
        let mut out = Vec::new();

        if let Some(z_size) = self.z {
            for x in 0..self.x {
                for y in 0..self.y {
                    for z in 0..z_size {
                        out.push(GridPosition::new_xyz(x, y, z));
                    }
                }
            }
        } else {
            for x in 0..self.x {
                for y in 0..self.y {
                    out.push(GridPosition::new_xy(x, y));
                }
            }
        }

        out
    }

    /// Get Position distance from border
    pub fn distance_from_border(&self, position: &GridPosition) -> u32 {
        *[
            *position.x(),
            self.x - *position.x() - 1,
            *position.y(),
            self.y - *position.y() - 1,
        ]
        .iter()
        .min()
        .unwrap()
    }

    /// Get Position distance from center.
    pub fn distance_from_center(&self, position: &GridPosition) -> u32 {
        if &self.center.0 < position.x() {
            position.x() - self.center.0
        } else {
            self.center.0 - position.x()
        }
        .min(if &self.center.1 < position.y() {
            position.y() - self.center.1
        } else {
            self.center.1 - position.y()
        })
    }

    fn calc_center_approx(x: u32, y: u32) -> (u32, u32) {
        (x / 2, y / 2)
    }
}

/// Basic two-dimensional GridMap.
///
/// Grid Map holds data of all objects inside, with their [`GridPosition`] for easy access and additional methods for
/// handling them.
///
/// Extend of created `GridMap` usage stems from additional traits that are implemented for collected objects.
pub struct GridMap2D<Data: TileData> {
    pub(crate) size: GridSize,
    pub(crate) tiles: Grid<Option<Data>>,
    pub(crate) layer: Option<u32>,
}

impl<Data: TileData> GridMap2D<Data> {
    /// Creates new, empty map of given size.
    pub fn new(size: GridSize) -> Self {
        Self {
            size,
            tiles: Grid::new(size.x as usize, size.y as usize),
            layer: None,
        }
    }

    /// Get tile at specified position.
    pub fn get_tile_at_position(&self, position: &GridPosition) -> Option<GridTileRef<Data>> {
        if !self.size.is_position_valid(position) {
            return None;
        }
        GridTileRef::maybe_new(
            *position,
            self.tiles
                .get(*position.x(), *position.y())
                .unwrap()
                .as_ref(),
        )
    }

    /// Get tile at specified position mutably.
    pub fn get_mut_tile_at_position(
        &mut self,
        position: &GridPosition,
    ) -> Option<GridTileRefMut<Data>> {
        if !self.size.is_position_valid(position) {
            return None;
        }
        GridTileRefMut::maybe_new(
            *position,
            self.tiles
                .get_mut(*position.x(), *position.y())
                .unwrap()
                .as_mut(),
        )
    }

    /// Insert tile. Its position will be determined based on information in [GridTile2D::grid_position]. If tile is
    /// present at that position already, it will be overwritten.
    pub fn insert_tile(&mut self, tile: GridTile<Data>) -> bool {
        if !self.size.is_position_valid(&tile.grid_position()) {
            return false;
        }
        let (x, y) = tile.grid_position().xy();
        let t = self.tiles.get_mut(x, y).unwrap();
        *t = Some(tile.into_inner());
        true
    }

    pub fn insert_data(&mut self, position: &GridPosition, data: Data) -> bool {
        if !self.size.is_position_valid(position) {
            return false;
        }
        let (x, y) = position.xy();
        let t = self.tiles.get_mut(x, y).unwrap();
        *t = Some(data);
        true
    }

    pub fn remove_tile_at_position(&mut self, position: &GridPosition) -> bool {
        if !self.size.is_position_valid(position) {
            return false;
        }
        if let Some(tile) = self.tiles.get_mut(*position.x(), *position.y()) {
            *tile = None;
        }
        true
    }

    pub fn size(&self) -> &GridSize {
        &self.size
    }

    /// Get tiles neighbouring the specified position.
    pub fn get_neighbours(&self, position: &GridPosition) -> Vec<GridTileRef<Data>> {
        GridDir::ALL_2D
            .iter()
            .filter_map(|direction| self.get_neighbour_at(position, direction))
            .collect::<Vec<_>>()
    }

    /// Get tile neighbouring the specified position at specified direction.
    pub fn get_neighbour_at(
        &self,
        position: &GridPosition,
        direction: &GridDir,
    ) -> Option<GridTileRef<Data>> {
        if let Some(position) = direction.march_step(position, &self.size) {
            return self.get_tile_at_position(&position);
        }
        None
    }

    pub fn get_mut_neighbour_at(
        &mut self,
        position: &GridPosition,
        direction: &GridDir,
    ) -> Option<GridTileRefMut<Data>> {
        if let Some(position) = direction.march_step(position, &self.size) {
            return self.get_mut_tile_at_position(&position);
        }
        None
    }

    /// Get positions of all tiles that are occupied within the GridMap
    pub fn get_all_positions(&self) -> Vec<GridPosition> {
        self.tiles
            .indexed_iter()
            .filter_map(|(pos, t)| {
                if t.is_some() {
                    Some((pos.0 as u32, pos.1 as u32))
                } else {
                    None
                }
            })
            .map(|(x, y)| GridPosition::new_xy(x, y))
            .collect::<Vec<GridPosition>>()
    }

    /// Get positions of all tiles that are in the border
    pub fn get_all_border_positions(&self, direction: &GridDir) -> Vec<GridPosition> {
        self.tiles
            .indexed_iter()
            .filter_map(|(pos, t)| {
                if t.is_some() {
                    let position = GridPosition::new_xy(pos.0 as u32, pos.1 as u32);
                    if self.get_neighbour_at(&position, direction).is_some() {
                        return Some(position);
                    }
                }
                None
            })
            .collect::<Vec<GridPosition>>()
    }

    pub fn get_all_empty_positions(&self) -> Vec<GridPosition> {
        self.tiles
            .indexed_iter()
            .filter_map(|(pos, t)| {
                if t.is_none() {
                    Some(GridPosition::new_xy(pos.0 as u32, pos.1 as u32))
                } else {
                    None
                }
            })
            .collect::<Vec<GridPosition>>()
    }

    pub fn iter_tiles(&self) -> impl Iterator<Item = GridTileRef<Data>> {
        self.tiles.indexed_iter().filter_map(|(pos, data)| {
            data.as_ref()
                .map(|d| GridTileRef::new(GridPosition::new_xy(pos.0 as u32, pos.1 as u32), d))
        })
    }

    /// Destroys the GridMap, returning all tiles with their position adjusted in relation to the `anchor_pos`.
    pub fn drain_remapped(mut self, anchor_pos: GridPosition) -> Vec<GridTile<Data>> {
        self.get_all_positions()
            .iter()
            .filter_map(|pos| {
                self.tiles
                    .get_mut(*pos.x(), *pos.y())
                    .unwrap()
                    .take()
                    .map(|data| GridTile::new(anchor_pos + *pos, data))
            })
            .collect()
    }

    /// Destroys the GridMap, returning all tiles.
    pub fn drain(mut self) -> Vec<GridTile<Data>> {
        self.get_all_positions()
            .iter()
            .filter_map(|pos| {
                self.tiles
                    .get_mut(*pos.x(), *pos.y())
                    .unwrap()
                    .take()
                    .map(|data| GridTile::new(*pos, data))
            })
            .collect()
    }

    /// Fills empty positions using constructor function.
    pub fn fill_empty_using(&mut self, func: fn(GridPosition) -> GridTile<Data>) {
        for position in self.get_all_empty_positions() {
            self.insert_tile(func(position));
        }
    }
}

impl<Data: TileData + Default> GridMap2D<Data> {
    pub fn fill_empty_with_default(&mut self) {
        for pos in self.get_all_empty_positions() {
            self.insert_data(&pos, Data::default());
        }
    }
}

impl<Data: TileData + Clone> GridMap2D<Data> {
    pub fn fill_empty_with(&mut self, tile: Data) {
        for pos in self.get_all_empty_positions() {
            self.insert_data(&pos, tile.clone());
        }
    }

    /// Get all tiles with their positions remapped according to `anchor_pos`, which is the `left-top` position.
    pub fn get_remapped(&self, anchor_pos: GridPosition) -> Vec<GridTile<Data>> {
        self.tiles
            .indexed_iter()
            .filter_map(|(pos, maybe_data)| {
                if let Some(data) = maybe_data {
                    let mut cloned = anchor_pos;
                    cloned.add_xy((pos.0 as u32, pos.1 as u32));
                    Some(GridTile::new(cloned, data.clone()))
                } else {
                    None
                }
            })
            .collect()
    }
}

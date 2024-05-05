use std::{cmp::Ordering, collections::VecDeque};

use rand::Rng;

use crate::{
    gen::collapse::{frequency::FrequencyHints, tile::CollapsibleTile},
    map::GridMap2D,
    tile::{identifiable::IdentifiableTile, GridTile2D},
    GridPos2D,
};

use super::{CollapseQueue, ResolverSelector};

/// Enum defining the starting point of the collapse wave.
#[derive(Default, Eq, PartialEq)]
pub enum PositionQueueStartingPoint {
    #[default]
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

/// Enum defining the direction in which the tiles will be collapsed.
#[derive(Default, Eq, PartialEq)]
pub enum PositionQueueDirection {
    #[default]
    Rowwise,
    Columnwise,
}

#[derive(Default)]
pub struct PositionQueue {
    starting_point: PositionQueueStartingPoint,
    progress_direction: PositionQueueDirection,
    positions: Vec<GridPos2D>,
    changed: bool,
}

impl PositionQueue {
    pub fn new(starting: PositionQueueStartingPoint, direction: PositionQueueDirection) -> Self {
        Self {
            starting_point: starting,
            progress_direction: direction,
            ..Default::default()
        }
    }

    pub fn sort_elements(&mut self) {
        let cmp_fun = match (&self.starting_point, &self.progress_direction) {
            (PositionQueueStartingPoint::UpLeft, PositionQueueDirection::Rowwise) => {
                compare_upleft_rowwise
            }
            (PositionQueueStartingPoint::UpLeft, PositionQueueDirection::Columnwise) => {
                compare_upleft_columnwise
            }
            (PositionQueueStartingPoint::UpRight, PositionQueueDirection::Rowwise) => {
                compare_upright_rowwise
            }
            (PositionQueueStartingPoint::UpRight, PositionQueueDirection::Columnwise) => {
                compare_upright_columnwise
            }
            (PositionQueueStartingPoint::DownLeft, PositionQueueDirection::Rowwise) => {
                compare_downleft_rowwise
            }
            (PositionQueueStartingPoint::DownLeft, PositionQueueDirection::Columnwise) => {
                compare_downleft_columnwise
            }
            (PositionQueueStartingPoint::DownRight, PositionQueueDirection::Rowwise) => {
                compare_downright_rowwise
            }
            (PositionQueueStartingPoint::DownRight, PositionQueueDirection::Columnwise) => {
                compare_downright_columnwise
            }
        };

        self.positions.sort_by(cmp_fun);
        self.positions.reverse();
    }
}

impl CollapseQueue for PositionQueue {
    fn get_next_position(&mut self) -> Option<GridPos2D> {
        if self.changed {
            self.sort_elements()
        }
        self.positions.pop()
    }

    fn initialize_queue(&mut self, tiles: &[CollapsibleTile]) {
        for tile in tiles {
            self.update_queue(tile)
        }
    }

    fn update_queue(&mut self, tile: &CollapsibleTile) {
        if !self.positions.contains(&tile.grid_position()) {
            self.positions.push(tile.grid_position());
        }
        self.changed = true;
    }

    fn len(&self) -> usize {
        self.positions.len()
    }

    fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }
}

impl ResolverSelector for PositionQueue {
    fn populate_inner_grid<R: Rng, InputTile: IdentifiableTile>(
        &mut self,
        _rng: &mut R,
        grid: &mut GridMap2D<CollapsibleTile>,
        positions: &[GridPos2D],
        frequency: &FrequencyHints<InputTile>,
    ) {
        let tiles = CollapsibleTile::new_from_frequency(positions, frequency);
        self.initialize_queue(&tiles);
        for tile in tiles {
            grid.insert_tile(tile);
        }
    }
}

// --- Comparison functions --- //
fn compare_upleft_columnwise(a: &(u32, u32), b: &(u32, u32)) -> Ordering {
    let cmp_a = a.0.cmp(&b.0);
    if cmp_a == Ordering::Equal {
        a.1.cmp(&b.1)
    } else {
        cmp_a
    }
}

fn compare_upleft_rowwise(a: &(u32, u32), b: &(u32, u32)) -> Ordering {
    let cmp_a = a.1.cmp(&b.1);
    if cmp_a == Ordering::Equal {
        a.0.cmp(&b.0)
    } else {
        cmp_a
    }
}

fn compare_upright_columnwise(a: &(u32, u32), b: &(u32, u32)) -> Ordering {
    let cmp_a = a.0.cmp(&b.0).reverse();
    if cmp_a == Ordering::Equal {
        a.1.cmp(&b.1)
    } else {
        cmp_a
    }
}

fn compare_upright_rowwise(a: &(u32, u32), b: &(u32, u32)) -> Ordering {
    let cmp_a = a.1.cmp(&b.1);
    if cmp_a == Ordering::Equal {
        a.0.cmp(&b.0)
    } else {
        cmp_a
    }
}

fn compare_downleft_columnwise(a: &(u32, u32), b: &(u32, u32)) -> Ordering {
    let cmp_a = a.0.cmp(&b.0);
    if cmp_a == Ordering::Equal {
        b.1.cmp(&a.1).reverse()
    } else {
        cmp_a
    }
}

fn compare_downleft_rowwise(a: &(u32, u32), b: &(u32, u32)) -> Ordering {
    let cmp_a = a.1.cmp(&b.1).reverse();
    if cmp_a == Ordering::Equal {
        b.0.cmp(&a.0)
    } else {
        cmp_a
    }
}

fn compare_downright_columnwise(a: &(u32, u32), b: &(u32, u32)) -> Ordering {
    let cmp_a = a.0.cmp(&b.0).reverse();
    if cmp_a == Ordering::Equal {
        b.1.cmp(&a.1).reverse()
    } else {
        cmp_a
    }
}

fn compare_downright_rowwise(a: &(u32, u32), b: &(u32, u32)) -> Ordering {
    let cmp_a = a.1.cmp(&b.1).reverse();
    if cmp_a == Ordering::Equal {
        b.0.cmp(&a.0).reverse()
    } else {
        cmp_a
    }
}

#[cfg(test)]
mod test {
    use crate::{
        gen::collapse::queue::position::compare_downleft_columnwise, gen_grid_positions_square,
    };

    #[test]
    fn check_sort_default() {
        let mut tiles = gen_grid_positions_square((0, 0), (5, 5));

        tiles.sort_by(compare_downleft_columnwise);

        println!("{tiles:?}");
    }
}

use std::{cmp::Ordering, collections::BTreeMap};

use rand::Rng;

use crate::tile::{GridPosition, GridTile, TileContainer};
use crate::gen::collapse::tile::CollapsibleData;
use crate::map::GridMap2D;

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
    positions: Vec<GridPosition>,
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
    fn get_next_position(&mut self) -> Option<GridPosition> {
        if self.changed {
            self.sort_elements()
        }
        self.positions.pop()
    }

    fn initialize_queue<Data: CollapsibleData>(&mut self, tiles: &[GridTile<Data>]) {
        for tile in tiles {
            self.update_queue(tile)
        }
    }

    fn update_queue<Tile, Data>(&mut self, tile: &Tile)
    where
        Tile: TileContainer + AsRef<Data>,
        Data: CollapsibleData,
    {
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
    fn populate_inner_grid<R: Rng, Data: CollapsibleData>(
        &mut self,
        _rng: &mut R,
        grid: &mut GridMap2D<Data>,
        positions: &[GridPosition],
        options_with_weights: BTreeMap<u64, u32>,
    ) {
        let tiles = Data::new_from_frequency(positions, options_with_weights);
        self.initialize_queue(&tiles);
        for tile in tiles {
            grid.insert_tile(tile);
        }
    }
}

// --- Comparison functions --- //
fn compare_upleft_columnwise(a: &GridPosition, b: &GridPosition) -> Ordering {
    let cmp_z = a.z().cmp(b.z());
    if cmp_z != Ordering::Equal {
        return cmp_z;
    }
    let cmp_a = a.x().cmp(b.x());
    if cmp_a == Ordering::Equal {
        a.y().cmp(b.y())
    } else {
        cmp_a
    }
}

fn compare_upleft_rowwise(a: &GridPosition, b: &GridPosition) -> Ordering {
    let cmp_z = a.z().cmp(b.z());
    if cmp_z != Ordering::Equal {
        return cmp_z;
    }
    let cmp_a = a.y().cmp(b.y());
    if cmp_a == Ordering::Equal {
        a.x().cmp(b.x())
    } else {
        cmp_a
    }
}

fn compare_upright_columnwise(a: &GridPosition, b: &GridPosition) -> Ordering {
    let cmp_z = a.z().cmp(b.z());
    if cmp_z != Ordering::Equal {
        return cmp_z;
    }
    let cmp_a = a.x().cmp(b.x()).reverse();
    if cmp_a == Ordering::Equal {
        a.y().cmp(b.y())
    } else {
        cmp_a
    }
}

fn compare_upright_rowwise(a: &GridPosition, b: &GridPosition) -> Ordering {
    let cmp_z = a.z().cmp(b.z());
    if cmp_z != Ordering::Equal {
        return cmp_z;
    }
    let cmp_a = a.y().cmp(b.y());
    if cmp_a == Ordering::Equal {
        a.x().cmp(b.x())
    } else {
        cmp_a
    }
}

fn compare_downleft_columnwise(a: &GridPosition, b: &GridPosition) -> Ordering {
    let cmp_z = a.z().cmp(b.z());
    if cmp_z != Ordering::Equal {
        return cmp_z;
    }
    let cmp_a = a.x().cmp(b.x());
    if cmp_a == Ordering::Equal {
        b.y().cmp(a.y()).reverse()
    } else {
        cmp_a
    }
}

fn compare_downleft_rowwise(a: &GridPosition, b: &GridPosition) -> Ordering {
    let cmp_z = a.z().cmp(b.z());
    if cmp_z != Ordering::Equal {
        return cmp_z;
    }
    let cmp_a = a.y().cmp(b.y()).reverse();
    if cmp_a == Ordering::Equal {
        b.x().cmp(a.x())
    } else {
        cmp_a
    }
}

fn compare_downright_columnwise(a: &GridPosition, b: &GridPosition) -> Ordering {
    let cmp_z = a.z().cmp(b.z());
    if cmp_z != Ordering::Equal {
        return cmp_z;
    }
    let cmp_a = a.x().cmp(b.x()).reverse();
    if cmp_a == Ordering::Equal {
        b.y().cmp(a.y()).reverse()
    } else {
        cmp_a
    }
}

fn compare_downright_rowwise(a: &GridPosition, b: &GridPosition) -> Ordering {
    let cmp_z = a.z().cmp(b.z());
    if cmp_z != Ordering::Equal {
        return cmp_z;
    }
    let cmp_a = a.y().cmp(b.y()).reverse();
    if cmp_a == Ordering::Equal {
        b.x().cmp(a.x()).reverse()
    } else {
        cmp_a
    }
}

#[cfg(test)]
mod test {
    use crate::{gen::collapse::queue::position::compare_downleft_columnwise, tile::GridPosition};

    #[test]
    fn check_sort_default() {
        let mut tiles = GridPosition::generate_rect_area(
            &GridPosition::new_xy(0, 0),
            &GridPosition::new_xy(5, 5),
        );

        tiles.sort_by(compare_downleft_columnwise);

        println!("{tiles:?}");
    }
}

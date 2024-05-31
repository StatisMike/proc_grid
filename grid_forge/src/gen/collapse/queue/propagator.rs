use crate::{
    gen::collapse::{AdjacencyTable, CollapsibleTileData},
    map::{GridDir, GridMap2D},
    tile::GridPosition,
};

use super::CollapseQueue;

pub(crate) struct PropagateItem {
    position: GridPosition,
    to_remove: u64,
}

impl PropagateItem {
    pub fn position(&self) -> &GridPosition {
        &self.position
    }

    pub fn to_remove(&self) -> &u64 {
        &self.to_remove
    }

    pub fn new(position: GridPosition, to_remove: u64) -> Self {
        Self {
            position,
            to_remove,
        }
    }
}

#[derive(Default)]
pub(crate) struct Propagator {
    inner: Vec<PropagateItem>,
}

impl Propagator {
    pub fn push(&mut self, item: PropagateItem) {
        self.inner.push(item);
    }

    pub fn pop(&mut self) -> Option<PropagateItem> {
        self.inner.pop()
    }

    pub(crate) fn propagate<T: CollapsibleTileData, Q: CollapseQueue>(
        &mut self,
        grid: &mut GridMap2D<T>,
        queue: &mut Q,
        adjacency: &AdjacencyTable,
    ) -> Result<(), GridPosition> {
        let size = *grid.size();
        while let Some(item) = self.inner.pop() {
            for direction in GridDir::ALL_2D {
                let pos_to_update = if let Some(pos) = direction.march_step(&item.position, &size) {
                    pos
                } else {
                    continue;
                };
                let mut tile = grid
                    .get_mut_tile_at_position(&pos_to_update)
                    .expect("cannot get cell to propagate!");
                for option_id in adjacency
                    .get_all_adjacencies_in_direction(item.to_remove(), &direction.opposite())
                {
                    match tile
                        .as_mut()
                        .decrement_ways_to_be_option(*option_id, *direction)
                    {
                        crate::gen::collapse::WaysToBeOptionOutcome::Eliminated => {
                            tile.as_mut().remove_option(*option_id);
                            if !tile.as_ref().have_options() {
                                return Err(pos_to_update);
                            }
                        }
                        _ => continue,
                    }

                    self.push(PropagateItem::new(pos_to_update, *option_id));

                    // Propagate the changes made to intermediate neighbours only if the queue is propagating
                    if queue.propagating() {
                        queue.update_queue(&tile);
                    }
                }
            }
        }

        Ok(())
    }
}

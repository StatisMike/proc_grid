use std::collections::{HashMap, HashSet};

use crate::{
    gen::collapse::{option::PerOptionData, CollapsibleTileData},
    map::{GridDir, GridMap2D},
    tile::{GridPosition, TileContainer},
};

use super::CollapseQueue;

pub struct PropagateItem {
    position: GridPosition,
    to_remove: usize,
}

impl PropagateItem {
    pub fn position(&self) -> &GridPosition {
        &self.position
    }

    pub fn to_remove(&self) -> usize {
        self.to_remove
    }

    pub fn new(position: GridPosition, to_remove: usize) -> Self {
        Self {
            position,
            to_remove,
        }
    }
}

#[derive(Default)]
pub struct Propagator {
    inner: Vec<PropagateItem>,
    reserved_removal: HashMap<GridPosition, Vec<usize>>,
}

impl Propagator {
    pub fn push_propagate(&mut self, item: PropagateItem) {
        self.inner.push(item);
    }

    pub(crate) fn propagate<Tile: CollapsibleTileData, Q: CollapseQueue>(
        &mut self,
        grid: &mut GridMap2D<Tile>,
        option_data: &PerOptionData,
        queue: &mut Q,
        collapsed_pos: Option<&GridPosition>,
    ) -> Result<(), GridPosition> {
        let mut tiles_to_update = HashSet::new();
        let size = *grid.size();
        while let Some((position, to_remove)) = self.pop_to_resolve(collapsed_pos) {
            for direction in GridDir::ALL_2D {
                let pos_to_update =
                    if let Some(pos) = direction.opposite().march_step(&position, &size) {
                        pos
                    } else {
                        continue;
                    };
                let mut tile = if let Some(tile) = grid.get_mut_tile_at_position(&pos_to_update) {
                    tile
                } else {
                    continue;
                };
                if tile.as_ref().is_collapsed() {
                    continue;
                }
                for option_idx in
                    option_data.get_all_enabled_in_direction(to_remove, direction.opposite())
                {
                    let binding = tile.as_mut();
                    let removed = binding
                        .mut_ways_to_be_option()
                        .decrement(*option_idx, *direction);
                    if removed {
                        binding.remove_option(option_data.get_weights(*option_idx));
                    }
                    if !binding.has_compatible_options() {
                        return Err(pos_to_update);
                    }
                    match (removed, queue.propagating()) {
                        (true, true) => {
                            self.push_propagate(PropagateItem::new(pos_to_update, *option_idx))
                        }
                        (true, false) => self.retain_removal(pos_to_update, *option_idx),
                        (false, _) => continue,
                    }
                    if removed && queue.needs_update_after_options_change() {
                        tiles_to_update.insert(tile.grid_position());
                    }
                }
            }
        }

        for pos in tiles_to_update {
            queue.update_queue(&grid.get_tile_at_position(&pos).unwrap());
        }

        Ok(())
    }

    fn pop_to_resolve(
        &mut self,
        collapsed_pos: Option<&GridPosition>,
    ) -> Option<(GridPosition, usize)> {
        if let Some(pos) = collapsed_pos {
            if let Some(option) = self.get_from_retained(pos) {
                return Some((*pos, option));
            }
        }
        if let Some(PropagateItem {
            position,
            to_remove,
        }) = self.inner.pop()
        {
            return Some((position, to_remove));
        }
        None
    }

    fn get_from_retained(&mut self, position: &GridPosition) -> Option<usize> {
        let options_to_remove = self.reserved_removal.get_mut(position)?;

        if let Some(option_to_remove) = options_to_remove.pop() {
            Some(option_to_remove)
        } else {
            self.reserved_removal.remove(position);
            None
        }
    }

    fn retain_removal(&mut self, position: GridPosition, option_to_remove: usize) {
        match self.reserved_removal.entry(position) {
            std::collections::hash_map::Entry::Occupied(mut e) => {
                e.get_mut().push(option_to_remove);
            }
            std::collections::hash_map::Entry::Vacant(e) => {
                e.insert(vec![option_to_remove]);
            }
        };
    }
}

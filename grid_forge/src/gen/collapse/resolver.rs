use std::collections::VecDeque;
use std::marker::PhantomData;

use crate::gen::adjacency::AdjacencyRules;
use crate::map::{GridDir, GridMap2D, GridSize};
use crate::tile::identifiable::builder::{IdentTileBuilder, TileBuilderError};
use crate::tile::{identifiable::IdentifiableTile, GridTile2D};
use crate::GridPos2D;

use super::frequency::FrequencyHints;
use super::queue::CollapseQueue;
use super::tile::CollapsibleTile;
use super::CollapseError;

use rand::Rng;

pub struct CollapsibleResolver<InputTile>
where
    InputTile: IdentifiableTile,
{
    pub(crate) inner: GridMap2D<CollapsibleTile>,
    tile_ids: Vec<u64>,
    tile_type: PhantomData<InputTile>,
}

impl<InputTile> CollapsibleResolver<InputTile>
where
    InputTile: IdentifiableTile,
{
    pub fn new(size: GridSize) -> Self {
        Self {
            inner: GridMap2D::new(size),
            tile_ids: Vec::new(),
            tile_type: PhantomData,
        }
    }

    pub fn fill_with_collapsed(&mut self, tile_id: u64, positions: &[GridPos2D]) {
        for position in positions {
            self.inner.insert_tile(CollapsibleTile::new_collapsed(*position, tile_id));
        }
    }

    pub fn all_positions(&self) -> Vec<GridPos2D> {
        self.inner.get_all_positions()
    }

    pub fn all_empty_positions(&self) -> Vec<GridPos2D> {
        self.inner.get_all_empty_positions()
    }

    pub fn generate<R, Queue>(
        &mut self,
        rng: &mut R,
        positions: &[GridPos2D],
        mut queue: Queue,
        frequencies: &FrequencyHints<InputTile>,
        adjacencies: &AdjacencyRules<InputTile>,
    ) -> Result<(), CollapseError>
    where
        R: Rng,
        Queue: CollapseQueue,
        InputTile: IdentifiableTile,
    {
        // Begin populating grid.
        let mut changed = VecDeque::<GridPos2D>::new();

        queue.populate_inner_grid(rng, &mut self.inner, positions, frequencies);

        for position in positions {
            if self.remove_tile_options(
                position, 
                adjacencies, 
                positions, 
                &changed,
            !queue.propagating())? {
                changed.push_back(*position);
            }
        }

        // Updating options if any have changed.
        if queue.needs_update_after_options_change() {
            for position in changed.iter() {
                queue.update_queue(self.inner.get_tile_at_position(position).unwrap());
            }

            // Propagating queue needs propagation at this point also
            if queue.propagating() {
                while let Some(position_changed) = changed.pop_front() {
                    self.propagate_from(position_changed, &mut queue, adjacencies, &mut changed)?;
                }
            }
        }

        // Progress with collapse.
        while let Some(next_position) = queue.get_next_position() {
            // Without propagation needs to remove options before collapse.
            if !queue.propagating() {
                self.remove_tile_options(
                    &next_position, 
                    adjacencies, 
                    &[], 
                    &changed,
                false)?;
            }

            let to_collapse = self.inner.get_mut_tile_at_position(&next_position).unwrap();
            let collapsed = to_collapse.collapse(rng)?;

            if collapsed {
                let collapsed_id = to_collapse.get_tile_id();
                if !self.tile_ids.contains(&collapsed_id) {
                    self.tile_ids.push(collapsed_id);
                }
            }

            // With propagation - propagate after collapse.
            if collapsed && queue.propagating() {
                changed.push_back(next_position);

                while let Some(position_changed) = changed.pop_front() {
                    self.propagate_from(position_changed, &mut queue, adjacencies, &mut changed)?;
                }
            }
        }

        Ok(())
    }

    fn remove_tile_options(
        &mut self,
        pos: &GridPos2D,
        adjacency: &AdjacencyRules<InputTile>,
        omit_positions_unless_changed: &[GridPos2D],
        changed: &VecDeque<GridPos2D>,
        collapsed_only: bool
    ) -> Result<bool, CollapseError>
    where
        InputTile: IdentifiableTile,
    {
        let tile = self
            .inner
            .get_tile_at_position(pos)
            .expect("no tile at given position");

        if *pos == (2,13) {
            print!("got it!");
        }

        // If tile is collapsed don't do anything.
        if tile.is_collapsed() {
            return Ok(false);
        }

        let mut options_to_remove = Vec::new();

        if tile.options_with_weights.is_empty() {
            return Err(CollapseError::new_options_empty(*pos));
        }

        // Check if option is valid for each direction.
        for dir in GridDir::ALL {
            if let Some(neighbour) = self.inner.get_neighbour_at(pos, dir) {
                if omit_positions_unless_changed.contains(&neighbour.grid_position())
                    && !changed.contains(&neighbour.grid_position())
                {
                    continue;
                }
                if neighbour.is_collapsed() {
                    for option in tile.options_with_weights.keys() {
                        if !adjacency.is_valid_raw(*option, neighbour.get_tile_id(), *dir) {
                            options_to_remove.push(*option);
                        }
                    }
                } else if !collapsed_only {
                    let neighbour_options = neighbour
                        .options_with_weights
                        .keys()
                        .copied()
                        .collect::<Vec<_>>();
                    for option in tile.options_with_weights.keys() {
                        if !adjacency.is_valid_raw_any(*option, &neighbour_options, *dir) {
                            options_to_remove.push(*option);
                        }
                    }
                }
            }
        }

        // Apply changed to options.
        if options_to_remove.is_empty() {
            Ok(false)
        } else {
            let tile = self
                .inner
                .get_mut_tile_at_position(pos)
                .expect("no tile at position");
            for option in options_to_remove {
                tile.remove_option(option);
                if !tile.have_options() {
                    return Err(CollapseError::new_options_empty(*pos));
                }
            }
            Ok(true)
        }
    }

    fn propagate_from<Queue>(
        &mut self,
        pos: GridPos2D,
        queue: &mut Queue,
        adjacency: &AdjacencyRules<InputTile>,
        changed: &mut VecDeque<GridPos2D>,
    ) -> Result<(), CollapseError>
    where
        Queue: CollapseQueue,
    {
        let tile = self
            .inner
            .get_tile_at_position(&pos)
            .expect("cant retrieve tile to propagate from");
        if tile.is_collapsed() {
            let tile_id = tile.get_tile_id();
            for direction in GridDir::ALL {
                if let Some(neighbour) = self.inner.get_mut_neighbour_at(&pos, direction) {
                    if neighbour.is_collapsed() {
                        continue;
                    }
                    if !neighbour
                        .resolve_options_neighbour_collapsed(
                            adjacency,
                            direction.opposite(),
                            tile_id,
                        )?
                        .is_empty()
                    {
                        queue.update_queue(neighbour);
                        if !changed.contains(&neighbour.grid_position()) {
                            changed.push_back(neighbour.grid_position());
                        }
                    }
                }
            }
        } else {
            let tile_options = tile
                .options_with_weights
                .keys()
                .copied()
                .collect::<Vec<_>>();
            for direction in GridDir::ALL {
                if let Some(neighbour) = self.inner.get_mut_neighbour_at(&pos, direction) {
                    if neighbour.is_collapsed() {
                        continue;
                    }
                    if !neighbour
                        .resolve_options_neighbour_uncollapsed(
                            adjacency,
                            direction.opposite(),
                            &tile_options,
                        )?
                        .is_empty()
                    {
                        queue.update_queue(neighbour);
                        if !changed.contains(&neighbour.grid_position()) {
                            changed.push_back(neighbour.grid_position());
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn build_grid<OutputTile, Builder>(
        &self,
        builder: &Builder,
    ) -> Result<GridMap2D<OutputTile>, TileBuilderError>
    where
        OutputTile: IdentifiableTile,
        Builder: IdentTileBuilder<OutputTile>,
    {
        builder.check_missing_tile_creators(&self.tile_ids)?;

        let mut grid = GridMap2D::new(*self.inner.size());

        for position in self.inner.get_all_positions() {
            let tile = self.inner.get_tile_at_position(&position).unwrap();
            if !tile.is_collapsed() {
                continue;
            }

            grid.insert_tile(builder.create_identifiable_tile(position, tile.get_tile_id()));
        }

        Ok(grid)
    }
}
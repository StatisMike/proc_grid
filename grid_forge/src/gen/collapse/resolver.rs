use std::collections::VecDeque;
use std::marker::PhantomData;

use crate::map::{GridDir, GridMap2D, GridSize};
use crate::tile::identifiable::builders::{IdentTileBuilder, TileBuilderError};
use crate::tile::identifiable::IdentifiableTileData;
use crate::tile::GridPosition;
use crate::tile::TileContainer;

use super::error::CollapseErrorKind;
use super::frequency::FrequencyHints;
use super::queue::CollapseQueue;
use super::rules::AdjacencyRules;
use super::tile::{CollapsibleData, CollapsibleTileData};
use super::CollapseError;

use rand::Rng;

pub struct CollapsibleResolver<Data>
where
    Data: IdentifiableTileData,
{
    pub(crate) inner: GridMap2D<CollapsibleTileData>,
    tile_ids: Vec<u64>,
    tile_type: PhantomData<Data>,
}

impl<Data> CollapsibleResolver<Data>
where
    Data: IdentifiableTileData,
{
    pub fn new(size: GridSize) -> Self {
        Self {
            inner: GridMap2D::new(size),
            tile_ids: Vec::new(),
            tile_type: PhantomData,
        }
    }

    pub fn fill_with_collapsed(&mut self, tile_id: u64, positions: &[GridPosition]) {
        for position in positions {
            self.inner
                .insert_tile(CollapsibleTileData::new_collapsed_tile(*position, tile_id));
        }
    }

    pub fn all_positions(&self) -> Vec<GridPosition> {
        self.inner.get_all_positions()
    }

    pub fn all_empty_positions(&self) -> Vec<GridPosition> {
        self.inner.get_all_empty_positions()
    }

    pub fn uncollapsed(&self) -> Vec<GridPosition> {
        self.inner
            .iter_tiles()
            .filter_map(|t| {
                if t.inner().is_collapsed() {
                    None
                } else {
                    Some(t.grid_position())
                }
            })
            .collect::<Vec<_>>()
    }

    pub fn generate<R, Queue>(
        &mut self,
        rng: &mut R,
        positions: &[GridPosition],
        mut queue: Queue,
        frequencies: &FrequencyHints<Data>,
        adjacencies: &AdjacencyRules<Data>,
    ) -> Result<(), CollapseError>
    where
        R: Rng,
        Queue: CollapseQueue,
        Data: IdentifiableTileData,
    {
        // Begin populating grid.
        let mut changed = VecDeque::<GridPosition>::new();

        queue.populate_inner_grid(
            rng,
            &mut self.inner,
            positions,
            frequencies.get_all_weights_cloned(),
        );

        for position in positions {
            if CollapseError::from_result(
                self.remove_tile_options(
                    position,
                    adjacencies,
                    positions,
                    &changed,
                    !queue.propagating(),
                ),
                CollapseErrorKind::Init,
            )? {
                changed.push_back(*position);
            }
        }

        // Updating options if any have changed.
        if queue.needs_update_after_options_change() {
            for position in changed.iter() {
                queue.update_queue(&self.inner.get_tile_at_position(position).unwrap());
            }

            // Propagating queue needs propagation at this point also
            if queue.propagating() {
                while let Some(position_changed) = changed.pop_front() {
                    CollapseError::from_result(
                        self.propagate_from(
                            position_changed,
                            &mut queue,
                            adjacencies,
                            &mut changed,
                        ),
                        CollapseErrorKind::Init,
                    )?;
                }
            }
        }

        // Progress with collapse.
        while let Some(next_position) = queue.get_next_position() {
            CollapseError::from_result(
                self.remove_tile_options(&next_position, adjacencies, &[], &changed, false),
                CollapseErrorKind::Collapse,
            )?;

            let mut to_collapse = self.inner.get_mut_tile_at_position(&next_position).unwrap();
            let collapsed = to_collapse.collapse(rng)?;

            if collapsed {
                let collapsed_id = to_collapse.as_ref().tile_type_id();
                if !self.tile_ids.contains(&collapsed_id) {
                    self.tile_ids.push(collapsed_id);
                }
            }

            // With propagation - propagate after collapse recursively.
            if collapsed && queue.propagating() {
                let collapsed_position = next_position;
                changed.push_back(next_position);

                while let Some(position_changed) = changed.pop_front() {
                    if !queue.in_propagaton_range(&collapsed_position, &position_changed) {
                        continue;
                    }
                    CollapseError::from_result(
                        self.propagate_from(
                            position_changed,
                            &mut queue,
                            adjacencies,
                            &mut changed,
                        ),
                        CollapseErrorKind::Propagation,
                    )?;
                }
            } else if !queue.propagating() {
                // Without propagation - update only direct neighbours.

                CollapseError::from_result(
                    self.propagate_from(
                        next_position,
                        &mut queue,
                        adjacencies,
                        &mut VecDeque::new(),
                    ),
                    CollapseErrorKind::NeighbourUpdate,
                )?;
            }
        }

        Ok(())
    }

    fn remove_tile_options(
        &mut self,
        pos: &GridPosition,
        adjacency: &AdjacencyRules<Data>,
        omit_positions_unless_changed: &[GridPosition],
        changed: &VecDeque<GridPosition>,
        collapsed_only: bool,
    ) -> Result<bool, GridPosition>
    where
        Data: IdentifiableTileData,
    {
        let tile = self
            .inner
            .get_tile_at_position(pos)
            .expect("no tile at given position");

        // If tile is collapsed don't do anything.
        if tile.inner().is_collapsed() {
            return Ok(false);
        }

        let mut options_to_remove = Vec::new();

        if tile.inner().options_with_weights.is_empty() {
            return Err(*pos);
        }

        // Check if option is valid for each direction.
        for dir in GridDir::ALL_2D {
            if let Some(neighbour) = self.inner.get_neighbour_at(pos, dir) {
                if omit_positions_unless_changed.contains(&neighbour.grid_position())
                    && !changed.contains(&neighbour.grid_position())
                {
                    continue;
                }
                if neighbour.inner().is_collapsed() {
                    for option in tile.inner().options_with_weights.keys() {
                        if !adjacency.is_valid_raw(*option, neighbour.as_ref().tile_type_id(), *dir)
                        {
                            options_to_remove.push(*option);
                        }
                    }
                } else if !collapsed_only {
                    let neighbour_options = neighbour
                        .inner()
                        .options_with_weights
                        .keys()
                        .copied()
                        .collect::<Vec<_>>();
                    for option in tile.inner().options_with_weights.keys() {
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
            let mut tile = self
                .inner
                .get_mut_tile_at_position(pos)
                .expect("no tile at position");
            for option in options_to_remove {
                tile.remove_option(option);
                if !tile.inner().have_options() {
                    return Err(*pos);
                }
            }
            Ok(true)
        }
    }

    fn propagate_from<Queue>(
        &mut self,
        pos: GridPosition,
        queue: &mut Queue,
        adjacency: &AdjacencyRules<Data>,
        changed: &mut VecDeque<GridPosition>,
    ) -> Result<(), GridPosition>
    where
        Queue: CollapseQueue,
    {
        let tile = self
            .inner
            .get_tile_at_position(&pos)
            .expect("cant retrieve tile to propagate from");
        if tile.inner().is_collapsed() {
            let tile_id = tile.as_ref().tile_type_id();
            for direction in GridDir::ALL_2D {
                if let Some(mut neighbour) = self.inner.get_mut_neighbour_at(&pos, direction) {
                    if neighbour.inner().is_collapsed() {
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
                        if queue.needs_update_after_options_change() {
                            queue.update_queue(&neighbour);
                        }

                        if !changed.contains(&neighbour.grid_position()) {
                            changed.push_back(neighbour.grid_position());
                        }
                    }
                }
            }
        } else {
            let tile_options = tile
                .inner()
                .options_with_weights
                .keys()
                .copied()
                .collect::<Vec<_>>();
            for direction in GridDir::ALL_2D {
                if let Some(mut neighbour) = self.inner.get_mut_neighbour_at(&pos, direction) {
                    if neighbour.as_ref().is_collapsed() {
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
                        if queue.needs_update_after_options_change() {
                            queue.update_queue(&neighbour);
                        }
                        if !changed.contains(&neighbour.grid_position()) {
                            changed.push_back(neighbour.grid_position());
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn build_grid<OutputData, Builder>(
        &self,
        builder: &Builder,
    ) -> Result<GridMap2D<OutputData>, TileBuilderError>
    where
        OutputData: IdentifiableTileData,
        Builder: IdentTileBuilder<OutputData>,
    {
        builder.check_missing_ids(&self.tile_ids)?;

        let mut grid = GridMap2D::new(*self.inner.size());

        for position in self.inner.get_all_positions() {
            let tile = self.inner.get_tile_at_position(&position).unwrap();
            if !tile.as_ref().is_collapsed() {
                continue;
            }

            grid.insert_tile(builder.build_tile_unchecked(position, tile.as_ref().tile_type_id()));
        }

        Ok(grid)
    }
}

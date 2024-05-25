use std::{collections::VecDeque, marker::PhantomData};

use rand::Rng;

use crate::{
    gen::collapse::{
        error::CollapseErrorKind, tile::CollapsibleData, CollapseError, CollapseQueue,
        CollapsibleTileData, EntrophyQueue, FrequencyHints, PositionQueue,
    },
    map::{GridDir, GridMap2D, GridSize},
    tile::{
        identifiable::{
            builders::{IdentTileBuilder, TileBuilderError},
            IdentifiableTileData,
        },
        GridPosition, TileContainer,
    },
};

use super::{
    frequency::{PatternAdjacencyRules, PatternFrequencyHints},
    tile::CollapsiblePatternTileData,
};

enum ResolverStage {}

pub struct OverlappingState<Queue: CollapseQueue> {
    queue: Queue,
}

pub struct OverlappingContext {}

pub struct OverlappingResolver<const PX: usize, const PY: usize, const PZ: usize, Data>
where
    Data: IdentifiableTileData,
{
    data_type: PhantomData<*const Data>,
}

impl<const PX: usize, const PY: usize, const PZ: usize, Data> OverlappingResolver<PX, PY, PZ, Data>
where
    Data: IdentifiableTileData,
{
    pub fn add_uncollapsed_positions<Queue, R>(
        rng: &mut R,
        grid: &mut CollapsibleGrid<CollapsiblePatternTileData<PX, PY, PZ>>,
        positions: &[GridPosition],
        adjacency: &PatternAdjacencyRules<PX, PY, PZ, Data>,
        frequencies: &PatternFrequencyHints<PX, PY, PZ, Data>,
        queue: &mut Queue,
    ) -> Result<(), CollapseError>
    where
        Queue: CollapseQueue,
        R: Rng,
    {
        // Begin populating grid.
        let mut changed = VecDeque::<GridPosition>::new();

        queue.populate_inner_grid(
            rng,
            &mut grid.grid,
            positions,
            frequencies.get_all_weights_cloned(),
        );

        for position in positions {
            if CollapseError::from_result(
                Self::remove_non_valid_pattern_options(
                    grid,
                    position,
                    adjacency,
                    &[],
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
                queue.update_queue(&grid.grid.get_tile_at_position(position).unwrap());
            }

            // Propagating queue needs propagation at this point also
            if queue.propagating() {
                while let Some(position_changed) = changed.pop_front() {
                    CollapseError::from_result(
                        Self::propagate_from(
                            grid,
                            position_changed,
                            queue,
                            adjacency,
                            &mut changed,
                        ),
                        CollapseErrorKind::Init,
                    )?;
                }
            }
        }
        Ok(())
    }

    fn remove_non_valid_pattern_options(
        grid: &mut CollapsibleGrid<CollapsiblePatternTileData<PX, PY, PZ>>,
        pos: &GridPosition,
        adjacency: &PatternAdjacencyRules<PX, PY, PZ, Data>,
        omit_positions_unless_changed: &[GridPosition],
        changed: &VecDeque<GridPosition>,
        collapsed_only: bool,
    ) -> Result<bool, GridPosition>
    where
        Data: IdentifiableTileData,
    {
        let tile = grid
            .grid
            .get_tile_at_position(pos)
            .expect("no tile at given position");

        // If tile is collapsed don't do anything.
        if tile.as_ref().is_collapsed() {
            return Ok(false);
        }

        let mut options_to_remove = Vec::new();

        if tile.as_ref().options_with_weights.is_empty() {
            return Err(*pos);
        }

        // Check if option is valid for each direction.
        for dir in GridDir::ALL_2D {
            if let Some(neighbour) = grid.grid.get_neighbour_at(pos, dir) {
                if omit_positions_unless_changed.contains(&neighbour.grid_position())
                    && !changed.contains(&neighbour.grid_position())
                {
                    continue;
                }
                if neighbour.inner().is_collapsed() {
                    for option in tile.inner().options_with_weights.keys() {
                        if !adjacency.as_ref().check_adjacency(
                            *option,
                            *dir,
                            neighbour.as_ref().pattern_id.unwrap(),
                        ) {
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
                        if !adjacency.as_ref().check_adjacency_any(
                            *option,
                            *dir,
                            &neighbour_options,
                        ) {
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
            let mut tile = grid
                .grid
                .get_mut_tile_at_position(pos)
                .expect("no tile at position");
            for option in options_to_remove {
                tile.as_mut().remove_option(option);
                if !tile.inner().have_options() {
                    return Err(*pos);
                }
            }
            Ok(true)
        }
    }

    fn propagate_from<Queue>(
        grid: &mut CollapsibleGrid<CollapsiblePatternTileData<PX, PY, PZ>>,
        pos: GridPosition,
        queue: &mut Queue,
        adjacency: &PatternAdjacencyRules<PX, PY, PZ, Data>,
        changed: &mut VecDeque<GridPosition>,
    ) -> Result<(), GridPosition>
    where
        Queue: CollapseQueue,
    {
        let tile = grid
            .grid
            .get_tile_at_position(&pos)
            .expect("cant retrieve tile to propagate from");
        if tile.inner().is_collapsed() {
            let pattern_id = tile.as_ref().pattern_id.unwrap();
            for direction in GridDir::ALL_2D {
                if let Some(mut neighbour) = grid.grid.get_mut_neighbour_at(&pos, direction) {
                    if neighbour.inner().is_collapsed() {
                        continue;
                    }
                    if !neighbour
                        .resolve_options_neighbour_collapsed(
                            adjacency,
                            direction.opposite(),
                            pattern_id,
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
                if let Some(mut neighbour) = grid.grid.get_mut_neighbour_at(&pos, direction) {
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
}

pub struct CollapsibleGrid<Data: CollapsibleData> {
    pub(crate) any_uncollapsed: bool,
    pub(crate) grid: GridMap2D<Data>,
}

impl<Data: CollapsibleData> CollapsibleGrid<Data> {
    pub fn new(size: GridSize) -> Self {
        Self {
            any_uncollapsed: false,
            grid: GridMap2D::new(size),
        }
    }

    pub fn any_uncollapsed(&self) -> bool {
        self.any_uncollapsed
    }

    pub fn add_collapsed(&mut self, positions: &[GridPosition], collapsed_tile_id: u64) {
        for pos in positions.iter() {
            self.grid
                .insert_tile(Data::new_collapsed_tile(*pos, collapsed_tile_id));
        }
    }

    pub fn build_grid<T: IdentifiableTileData, B: IdentTileBuilder<T>>(
        &self,
        builder: &B,
    ) -> Result<GridMap2D<T>, TileBuilderError> {
        let mut map = GridMap2D::new(self.grid.size);

        for pos in self.grid.get_all_positions() {
            let tile = self.grid.get_tile_at_position(&pos).unwrap();

            if tile.as_ref().is_collapsed() {
                map.insert_tile(builder.build_tile(pos, tile.as_ref().tile_type_id())?);
            }
        }

        Ok(map)
    }
}

impl GridMap2D<CollapsibleTileData> {}

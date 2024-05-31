use std::collections::{BTreeMap, HashSet, VecDeque};
use std::marker::PhantomData;

use rand::distributions::Distribution;
use rand::Rng;

use crate::gen::collapse::error::{CollapseError, CollapseErrorKind};
use crate::gen::collapse::queue::CollapseQueue;
use crate::gen::collapse::tile::private::Sealed;
use crate::gen::collapse::tile::CollapsibleTileData;
use crate::gen::collapse::Propagator;
use crate::map::{GridDir, GridMap2D, GridSize};

use crate::tile::identifiable::builders::{IdentTileBuilder, TileBuilderError};
use crate::tile::identifiable::collection::IdentTileCollection;
use crate::tile::identifiable::IdentifiableTileData;
use crate::tile::{GridPosition, TileContainer};

use super::analyzer::{AdjacencyRules, Analyzer, FrequencyHints};
use super::pattern::{OverlappingPattern, PatternCollection};
use super::tile::CollapsiblePattern;

pub struct Resolver<P, Data>
where
    P: OverlappingPattern,
    Data: IdentifiableTileData,
{
    grid: CollapsibleGrid<CollapsiblePattern<P>>,
    subscriber: Option<Box<dyn PatternCollapseSubscriber>>,
    data_type: PhantomData<*const Data>,
}

impl<P, Data> Resolver<P, Data>
where
    P: OverlappingPattern,
    Data: IdentifiableTileData,
{
    pub fn new(size: GridSize) -> Self {
        Self {
            grid: CollapsibleGrid::new(size),
            subscriber: None,
            data_type: PhantomData,
        }
    }

    pub fn with_subscriber(mut self, subscriber: Box<dyn PatternCollapseSubscriber>) -> Self {
        self.subscriber = Some(subscriber);
        self
    }

    pub fn generate_analyzer<R, Queue>(
        &mut self,
        rng: &mut R,
        positions: &[GridPosition],
        queue: Queue,
        analyzer: &Analyzer<P, Data>,
    ) -> Result<(), CollapseError>
    where
        R: Rng,
        Queue: CollapseQueue,
    {
        self.generate(
            rng,
            positions,
            queue,
            analyzer.get_collection(),
            analyzer.get_frequency(),
            analyzer.get_adjacency(),
        )
    }

    pub fn generate<R, Queue>(
        &mut self,
        rng: &mut R,
        positions: &[GridPosition],
        mut queue: Queue,
        collection: &PatternCollection<P>,
        frequency: &FrequencyHints<P, Data>,
        adjacency: &AdjacencyRules<P, Data>,
    ) -> Result<(), CollapseError>
    where
        R: Rng,
        Queue: CollapseQueue,
    {
        let mut propagator = Propagator::default();
        
        // Begin populating grid.
        let mut changed = VecDeque::<GridPosition>::new();

        if self.grid.need_pattern_population() {
            CollapseError::from_result(
                self.grid.populate_patterns(rng, collection, frequency),
                CollapseErrorKind::Init,
            )?;
            self.grid.add_to_queue(&mut queue);
        }

        queue.populate_inner_grid(
            rng,
            &mut self.grid.grid,
            positions,
            adjacency.inner(),
            frequency.get_all_weights_cloned(),
        );

        for position in positions {
            if CollapseError::from_result(
                Self::remove_non_valid_pattern_options(
                    &mut self.grid,
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
                queue.update_queue(&self.grid.grid.get_tile_at_position(position).unwrap());
            }

            // Propagating queue needs propagation at this point also
            if queue.propagating() {
                while let Some(position_changed) = changed.pop_front() {
                    CollapseError::from_result(
                        Self::propagate_from(
                            &mut self.grid,
                            position_changed,
                            &mut queue,
                            adjacency,
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
                Self::remove_non_valid_pattern_options(
                    &mut self.grid,
                    &next_position,
                    adjacency,
                    &[],
                    &changed,
                    false,
                ),
                CollapseErrorKind::Collapse,
            )?;

            let mut to_collapse = self
                .grid
                .grid
                .get_mut_tile_at_position(&next_position)
                .unwrap();
            let collapsed = to_collapse.collapse(rng)?;

            if collapsed {
                let pattern_id = to_collapse.as_ref().pattern_id.unwrap();
                let tile_type_id = collection
                    .get_tile_data(&pattern_id)
                    .unwrap()
                    .tile_type_id();
                to_collapse.as_mut().tile_type_id = Some(tile_type_id);
                self.grid.insert_tile_type_id(tile_type_id);
                if let Some(subscriber) = self.subscriber.as_mut() {
                    subscriber
                        .as_mut()
                        .on_collapse(&next_position, tile_type_id, pattern_id);
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
                        Self::propagate_from(
                            &mut self.grid,
                            position_changed,
                            &mut queue,
                            adjacency,
                            &mut changed,
                        ),
                        CollapseErrorKind::Propagation,
                    )?;
                }
            } else if !queue.propagating() {
                // Without propagation - update only direct neighbours.

                CollapseError::from_result(
                    Self::propagate_from(
                        &mut self.grid,
                        next_position,
                        &mut queue,
                        adjacency,
                        &mut VecDeque::new(),
                    ),
                    CollapseErrorKind::NeighbourUpdate,
                )?;
            }
        }

        Ok(())
    }

    fn remove_non_valid_pattern_options(
        grid: &mut CollapsibleGrid<CollapsiblePattern<P>>,
        pos: &GridPosition,
        adjacency: &AdjacencyRules<P, Data>,
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
                            option,
                            dir,
                            &neighbour.as_ref().pattern_id.unwrap(),
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
                        if !adjacency
                            .as_ref()
                            .check_adjacency_any(option, dir, &neighbour_options)
                        {
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
        grid: &mut CollapsibleGrid<CollapsiblePattern<P>>,
        pos: GridPosition,
        queue: &mut Queue,
        adjacency: &AdjacencyRules<P, Data>,
        changed: &mut VecDeque<GridPosition>,
    ) -> Result<(), GridPosition>
    where
        Queue: CollapseQueue,
    {
        let tile = grid
            .grid
            .get_tile_at_position(&pos)
            .expect("cant retrieve tile to propagate from");
        if tile.inner().pattern_id.is_some() {
            let pattern_id = tile.as_ref().pattern_id.unwrap();
            for direction in GridDir::ALL_2D {
                if let Some(mut neighbour) = grid.grid.get_mut_neighbour_at(&pos, direction) {
                    if changed.contains(&neighbour.grid_position()) {
                        continue;
                    }
                    if neighbour.inner().pattern_id.is_some() {
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
                    if neighbour.as_ref().pattern_id.is_some() {
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

    pub fn build_grid<OutData: IdentifiableTileData, B: IdentTileBuilder<OutData>>(
        &self,
        builder: &B,
    ) -> Result<GridMap2D<OutData>, TileBuilderError> {
        self.grid.build_grid(builder)
    }
}

pub struct CollapsibleGrid<Data: CollapsibleTileData> {
    pub(crate) any_uncollapsed: bool,
    pub(crate) collapsed_added: HashSet<GridPosition>,
    pub(crate) grid: GridMap2D<Data>,
    tile_type_ids: HashSet<u64>,
}

impl<Data: CollapsibleTileData> CollapsibleGrid<Data> {
    pub fn new(size: GridSize) -> Self {
        Self {
            any_uncollapsed: false,
            collapsed_added: HashSet::new(),
            grid: GridMap2D::new(size),
            tile_type_ids: HashSet::new(),
        }
    }

    pub fn any_uncollapsed(&self) -> bool {
        self.any_uncollapsed
    }

    pub fn tile_type_ids(&self) -> &HashSet<u64> {
        &self.tile_type_ids
    }

    pub fn insert_tile_type_id(&mut self, tile_type_id: u64) {
        self.tile_type_ids.insert(tile_type_id);
    }

    pub fn add_collapsed(&mut self, positions: &[GridPosition], collapsed_tile_id: u64) {
        self.tile_type_ids.insert(collapsed_tile_id);
        for pos in positions.iter() {
            self.grid
                .insert_tile(Data::new_collapsed_tile(*pos, collapsed_tile_id));
            self.collapsed_added.insert(*pos);
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

impl<P: OverlappingPattern> CollapsibleGrid<CollapsiblePattern<P>> {
    pub fn need_pattern_population(&self) -> bool {
        !self.collapsed_added.is_empty()
    }

    pub fn add_to_queue<Queue: CollapseQueue>(&self, queue: &mut Queue) {
        for position in self.collapsed_added.iter() {
            let tile = self.grid.get_tile_at_position(position).unwrap();
            if tile.as_ref().pattern_id.is_none() {
                queue.update_queue(&tile);
            }
        }
    }

    pub fn populate_patterns<Data: IdentifiableTileData, R: Rng>(
        &mut self,
        rng: &mut R,
        collection: &PatternCollection<P>,
        frequency: &FrequencyHints<P, Data>,
    ) -> Result<(), GridPosition> {
        let entrophy_uniform = CollapsiblePattern::<P>::entrophy_uniform();

        for position in self.collapsed_added.iter() {
            let mut possible_patterns = Vec::new();
            let tile_type_id = self
                .grid
                .get_tile_at_position(position)
                .unwrap()
                .as_ref()
                .tile_type_id();
            'pat_loop: for pattern in collection.get_patterns_for_tile(tile_type_id) {
                for pos_to_check in P::secondary_tile_positions(position) {
                    if let Some(Some(tile_type_id)) = self
                        .grid
                        .get_tile_at_position(&pos_to_check)
                        .map(|t| t.as_ref().tile_type_id)
                    {
                        if tile_type_id != pattern.get_id_for_pos(position, &pos_to_check) {
                            continue 'pat_loop;
                        }
                    }
                }
                possible_patterns.push(pattern.pattern_id());
            }
            if possible_patterns.is_empty() {
                return Err(*position);
            }

            let mut tile = self.grid.get_mut_tile_at_position(position).unwrap();

            let mut options_with_weights = BTreeMap::new();

            for option_id in possible_patterns {
                options_with_weights.insert(option_id, frequency.get_weight_for_pattern(option_id));
            }

            tile.as_mut()
                .set_weights(options_with_weights, entrophy_uniform.sample(rng));
        }

        Ok(())
    }
}

pub trait PatternCollapseSubscriber {
    fn on_collapse(&mut self, position: &GridPosition, tile_type_id: u64, pattern_id: u64);
}

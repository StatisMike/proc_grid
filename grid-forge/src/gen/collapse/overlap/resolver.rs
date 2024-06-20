use std::marker::PhantomData;

use rand::Rng;

use crate::gen::collapse::error::{CollapseError, CollapseErrorKind};
use crate::gen::collapse::grid::private::Sealed;
use crate::gen::collapse::overlap::CollapsiblePattern;
use crate::gen::collapse::queue::CollapseQueue;
use crate::gen::collapse::tile::CollapsibleTileData;
use crate::gen::collapse::{EntrophyQueue, PositionQueue, PropagateItem, Propagator};

use crate::tile::identifiable::collection::IdentTileCollection;
use crate::tile::identifiable::IdentifiableTileData;
use crate::tile::GridPosition;

use super::pattern::OverlappingPattern;
use super::CollapsiblePatternGrid;

pub struct Resolver<P, Data>
where
    P: OverlappingPattern,
    Data: IdentifiableTileData,
{
    subscriber: Option<Box<dyn Subscriber>>,
    tile_type: PhantomData<(P, Data)>,
}

impl<P, Data> Default for Resolver<P, Data>
where
    P: OverlappingPattern,
    Data: IdentifiableTileData,
{
    fn default() -> Self {
        Self {
            subscriber: None,
            tile_type: PhantomData,
        }
    }
}

impl<P, Data> Resolver<P, Data>
where
    P: OverlappingPattern,
    Data: IdentifiableTileData,
{
    pub fn with_subscriber(mut self, subscriber: Box<dyn Subscriber>) -> Self {
        self.subscriber = Some(subscriber);
        self
    }

    /// Retrieve the subscriber attached to the resolver.
    pub fn retrieve_subscriber(&mut self) -> Option<Box<dyn Subscriber>> {
        self.subscriber.take()
    }

    pub fn generate_entrophy<R>(
        &mut self,
        mut grid: CollapsiblePatternGrid<P, Data>,
        rng: &mut R,
        positions: &[GridPosition],
    ) -> Result<CollapsiblePatternGrid<P, Data>, CollapseError>
    where
        R: Rng,
    {
        use crate::gen::collapse::queue::private::Sealed as _;
        use crate::gen::collapse::tile::private::Sealed as _;

        let mut iter = 0;
        let mut queue = EntrophyQueue::default();

        if let Some(subscriber) = self.subscriber.as_mut() {
            subscriber.on_generation_start();
        }

        let mut propagator = Propagator::default();

        queue.populate_inner_grid(rng, &mut grid.pattern_grid, positions, &grid.option_data);

        for initial_propagate in grid._get_initial_propagate_items(positions) {
            propagator.push_propagate(initial_propagate);
        }

        CollapseError::from_result(
            propagator.propagate(&mut grid.pattern_grid, &grid.option_data, &mut queue),
            CollapseErrorKind::Init,
            iter,
        )?;

        while let Some(collapse_position) = queue.get_next_position() {
            let mut to_collapse = grid
                .pattern_grid
                .get_mut_tile_at_position(&collapse_position)
                .unwrap();
            // skip collapsed.
            if to_collapse.as_ref().is_collapsed() {
                continue;
            }

            if !to_collapse.as_ref().has_compatible_options() {
                return Err(CollapseError::new(
                    collapse_position,
                    CollapseErrorKind::Collapse,
                    iter,
                ));
            }

            let removed_options = to_collapse
                .as_mut()
                .collapse_gather_removed(rng, &grid.option_data);
            let collapsed_idx = to_collapse.as_ref().collapse_idx().unwrap();

            if let Some(subscriber) = self.subscriber.as_mut() {
                let pattern_id = grid.option_data.get_tile_type_id(&collapsed_idx).unwrap();
                let collapsed_id = grid
                    .patterns
                    .get_tile_data(&pattern_id)
                    .unwrap()
                    .tile_type_id();

                subscriber.on_collapse(&collapse_position, collapsed_id, pattern_id);
            }

            for removed_option in removed_options.into_iter() {
                propagator.push_propagate(PropagateItem::new(collapse_position, removed_option))
            }

            CollapseError::from_result(
                propagator.propagate(&mut grid.pattern_grid, &grid.option_data, &mut queue),
                CollapseErrorKind::Propagation,
                iter,
            )?;
            iter += 1;
        }

        Ok(grid)
    }

    pub fn generate_position<R>(
        &mut self,
        mut grid: CollapsiblePatternGrid<P, Data>,
        rng: &mut R,
        position: &[GridPosition],
        mut queue: PositionQueue,
    ) -> Result<CollapsiblePatternGrid<P, Data>, CollapseError>
    where
        R: Rng,
    {
        use crate::gen::collapse::queue::private::Sealed as _;
        use crate::gen::collapse::tile::private::Sealed as _;
        let mut iter = 0;

        if let Some(subscriber) = self.subscriber.as_mut() {
            subscriber.on_generation_start();
        }

        queue.populate_inner_grid(rng, &mut grid.pattern_grid, position, &grid.option_data);

        while let Some(collapse_position) = queue.get_next_position() {
            let to_collapse = grid
                .pattern_grid
                .get_tile_at_position(&collapse_position)
                .unwrap();
            // skip collapsed.
            if to_collapse.as_ref().is_collapsed() {
                continue;
            }

            if !to_collapse.as_ref().has_compatible_options()
                || !CollapsiblePattern::purge_incompatible_options(
                    &mut grid.pattern_grid,
                    &collapse_position,
                    &grid.option_data,
                )
            {
                return Err(CollapseError::new(
                    collapse_position,
                    CollapseErrorKind::Collapse,
                    iter,
                ));
            }

            let mut to_collapse = grid
                .pattern_grid
                .get_mut_tile_at_position(&collapse_position)
                .unwrap();

            to_collapse.as_mut().collapse_basic(rng, &grid.option_data);
            let collapsed_idx = to_collapse.as_ref().collapse_idx().unwrap();
            CollapsiblePattern::purge_options_for_neighbours(
                &mut grid.pattern_grid,
                collapsed_idx,
                &collapse_position,
                &grid.option_data,
            );

            if let Some(subscriber) = self.subscriber.as_mut() {
                let pattern_id = grid.option_data.get_tile_type_id(&collapsed_idx).unwrap();
                let collapsed_id = grid
                    .patterns
                    .get_tile_data(&pattern_id)
                    .unwrap()
                    .tile_type_id();

                subscriber.on_collapse(&collapse_position, collapsed_id, pattern_id);
            }
            iter += 1;
        }
        Ok(grid)
    }
}

/// When applied to the struct allows injecting it into [`overlap::Resolver`](Resolver) to react on each tile being collapsed.
pub trait Subscriber {
    /// Called when the generation process starts.
    fn on_generation_start(&mut self) {
        // no-op
    }

    /// Called when a tile is collapsed.
    fn on_collapse(&mut self, position: &GridPosition, tile_type_id: u64, pattern_id: u64);
}

/// Event in the history of tile generation process.
#[derive(Debug, Clone)]
pub struct CollapseHistoryItem {
    pub position: GridPosition,
    pub tile_type_id: u64,
    pub pattern_id: u64,
}

/// Simple subscriber to collect history of tile generation process.
///
/// Every new generation began by resolver will clear the history.
#[derive(Debug, Clone, Default)]
pub struct CollapseHistorySubscriber {
    history: Vec<CollapseHistoryItem>,
}

impl CollapseHistorySubscriber {
    /// Returns history of tile generation process.
    pub fn history(&self) -> &[CollapseHistoryItem] {
        &self.history
    }
}

impl Subscriber for CollapseHistorySubscriber {
    fn on_generation_start(&mut self) {
        self.history.clear();
    }

    fn on_collapse(&mut self, position: &GridPosition, tile_type_id: u64, pattern_id: u64) {
        self.history.push(CollapseHistoryItem {
            position: *position,
            tile_type_id,
            pattern_id,
        });
    }
}

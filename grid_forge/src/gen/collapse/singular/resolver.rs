use std::any::Any;
use std::marker::PhantomData;

use crate::gen::collapse::grid::private::Sealed;
use crate::gen::collapse::grid::CollapsibleGrid;
use crate::gen::collapse::{
    CollapsibleTileData, EntrophyQueue, PositionQueue, PropagateItem, Propagator,
};
use crate::tile::identifiable::collection::IdentTileCollection;
use crate::tile::identifiable::IdentifiableTileData;
use crate::tile::GridPosition;

use crate::gen::collapse::error::{CollapseError, CollapseErrorKind};
use crate::gen::collapse::queue::CollapseQueue;

use super::{CollapsibleTile, CollapsibleTileGrid};

use rand::Rng;

/// Resolver of the singular collapsible procedural algorithm.
/// 
/// It uses either [`EntrophyQueue`] or [`PositionQueue`] to process the option collapsing process of the [`CollapsibleTileGrid`], 
/// additionally providing an option to subscribe to the collapse process via [`singular::Subscriber`](Subscriber).
pub struct Resolver<Data>
where
    Data: IdentifiableTileData,
{
    subscriber: Option<Box<dyn Subscriber>>,
    tile_type: PhantomData<Data>,
}

impl<Data> Default for Resolver<Data>
where
    Data: IdentifiableTileData,
{
    fn default() -> Self {
        Self {
            subscriber: None,
            tile_type: PhantomData,
        }
    }
}

impl<Data> Resolver<Data>
where
    Data: IdentifiableTileData,
{
    /// Attach a subscriber to the resolver. The subscriber will be notified of each tile being collapsed.
    pub fn with_subscriber(mut self, subscriber: Box<dyn Subscriber>) -> Self {
        self.subscriber = Some(subscriber);
        self
    }

    /// Retrieve the subscriber attached to the resolver.
    pub fn retrieve_subscriber(&mut self) -> Option<Box<dyn Subscriber>> {
        self.subscriber.take()
    }

    /// Collapse the [`CollapsibleTileGrid`] using [`EntrophyQueue`].
    /// 
    /// Contrary to [`generate_position`](Self::generate_position), this method don't require providing the precreated
    /// queue, as it don't allow for any configuration - it will always collapse the tile with the lowest entrophy next.
    ///
    /// # Arguments
    /// * `grid` - [`CollapsibleTileGrid`] to be processed. All non-collapsed tiles provided within will be
    /// removed on the beginning of the process.
    /// * `rng` - [`Rng`] to be used for randomness.
    /// * `positions` - [`GridPosition`]s to be collapsed. If any collapsed tile is present inside the provided `grid`
    /// at one of the positions provided, the tile will be overwritten with uncollapsed one.
    /// 
    /// Provided `grid` can be translated into either a [`CollapsedGrid`](crate::gen::collapse::grid::CollapsedGrid)
    /// or [`GridMap2D`](crate::map::GridMap2D) of some [`IdentifiableTileData`] after the process.
    pub fn generate_entrophy<R>(
        &mut self,
        grid: &mut CollapsibleTileGrid<Data>,
        rng: &mut R,
        positions: &[GridPosition],
    ) -> Result<(), CollapseError>
    where
        R: Rng,
    {
        use crate::gen::collapse::queue::private::Sealed as _;
        use crate::gen::collapse::tile::private::Sealed as _;

        let mut iter = 0;
        let mut queue = EntrophyQueue::default();
        let mut propagator = Propagator::default();

        if let Some(subscriber) = self.subscriber.as_mut() {
            subscriber.on_generation_start();
        }

        grid.remove_uncollapsed();

        queue.populate_inner_grid(rng, &mut grid.grid, positions, &grid.option_data);

        for initial_propagate in grid._get_initial_propagate_items(positions) {
            propagator.push_propagate(initial_propagate);
        }

        CollapseError::from_result(
            propagator.propagate(&mut grid.grid, &grid.option_data, &mut queue),
            CollapseErrorKind::Init,
            iter,
        )?;

        // Progress with collapse.
        while let Some(collapse_position) = queue.get_next_position() {
            let mut to_collapse = grid
                .grid
                .get_mut_tile_at_position(&collapse_position)
                .unwrap();
            // skip collapsed;
            if to_collapse.as_ref().is_collapsed() {
                continue;
            }
            if !to_collapse.as_ref().has_compatible_options() {
                return Err(CollapseError::new(
                    collapse_position,
                    CollapseErrorKind::Collapse,
                    iter
                ));
            }
            let Some(removed_options) = to_collapse.as_mut().collapse(rng, &grid.option_data)
            else {
                return Err(CollapseError::new(
                    collapse_position,
                    CollapseErrorKind::Collapse,
                    iter
                ));
            };
            let collapsed_idx = to_collapse.as_ref().collapse_idx().unwrap();
            if let Some(subscriber) = self.subscriber.as_mut() {
                let collapsed_id = grid
                    ._option_data()
                    .get_tile_type_id(&collapsed_idx)
                    .unwrap();
                subscriber
                    .as_mut()
                    .on_collapse(&collapse_position, collapsed_id);
            }
            for removed_option in removed_options.into_iter() {
                propagator.push_propagate(PropagateItem::new(collapse_position, removed_option))
            }
            CollapseError::from_result(
                propagator.propagate(&mut grid.grid, &grid.option_data, &mut queue),
                CollapseErrorKind::Propagation,
                iter,
            )?;
            iter += 1;
        }

        Ok(())
    }


    pub fn generate_position<R>(
        &mut self,
        grid: &mut CollapsibleTileGrid<Data>,
        rng: &mut R,
        positions: &[GridPosition],
        mut queue: PositionQueue,
    ) -> Result<(), CollapseError>
    where
        R: Rng,
    {
        use crate::gen::collapse::queue::private::Sealed as _;
        use crate::gen::collapse::tile::private::Sealed as _;
        let mut iter = 0;

        if let Some(subscriber) = self.subscriber.as_mut() {
            subscriber.on_generation_start();
        }

        grid.remove_uncollapsed();

        queue.populate_inner_grid(rng, &mut grid.grid, positions, &grid.option_data);

        // Progress with collapse.
        while let Some(collapse_position) = queue.get_next_position() {
            let to_collapse = grid.grid.get_tile_at_position(&collapse_position).unwrap();
            // skip collapsed;
            if to_collapse.as_ref().is_collapsed() {
                continue;
            }
            // Make sure that the tile has at leas option, and purge them based on the direct neighbours.
            if !to_collapse.as_ref().has_compatible_options()
                || !CollapsibleTile::purge_incompatible_options(
                    &mut grid.grid,
                    &collapse_position,
                    &grid.option_data,
                )
            {
                return Err(CollapseError::new(
                    collapse_position,
                    CollapseErrorKind::Collapse,
                    iter
                ));
            };

            let mut to_collapse = grid
                .grid
                .get_mut_tile_at_position(&collapse_position)
                .unwrap();
            to_collapse.as_mut().collapse_basic(rng, &grid.option_data);

            let collapsed_idx = to_collapse.as_ref().collapse_idx().unwrap();
            // Purge options for the neighbours. This step is not required for the generation to be sound at the end,
            // but it increases the success rate of the process greatly at the relatively small performance cost.
            CollapsibleTile::purge_options_for_neighbours(
                &mut grid.grid,
                collapsed_idx,
                &collapse_position,
                &grid.option_data,
            );

            if let Some(subscriber) = self.subscriber.as_mut() {
                let collapsed_id = grid
                    ._option_data()
                    .get_tile_type_id(&collapsed_idx)
                    .unwrap();
                subscriber
                    .as_mut()
                    .on_collapse(&collapse_position, collapsed_id);
            }
            iter += 1;
        }
        Ok(())
    }
}

/// When applied to the struct allows injecting it into [`singular::Resolver`](Resolver) to react on each tile being collapsed.
pub trait Subscriber: Any {
    /// Called when the generation process starts. No-op by default, should be overridden to clear the state of the subcscriber
    /// if it retains any state.
    fn on_generation_start(&mut self) {
        // no-op
    }

    /// Called when a tile is collapsed.
    fn on_collapse(&mut self, position: &GridPosition, tile_type_id: u64);

    /// To retrieve the concrete subscriber type from [`singular::Resolver`](Resolver).
    fn as_any(&self) -> &dyn Any;
}

/// Event in the history of tile generation process, containing the [`GridPosition`] of the tile alongside its collapsed
/// `tile_type_id`.
#[derive(Debug, Clone)]
pub struct CollapseHistoryItem {
    pub position: GridPosition,
    pub tile_type_id: u64,
}

impl From<crate::gen::collapse::overlap::CollapseHistoryItem> for CollapseHistoryItem {
    fn from(item: crate::gen::collapse::overlap::CollapseHistoryItem) -> Self {
        Self {
            position: item.position,
            tile_type_id: item.tile_type_id,
        }
    }
}

/// Simple subscriber to collect history of tile generation process.
///
/// Every new generation began by the resolver will clear the history.
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

    fn on_collapse(&mut self, position: &GridPosition, tile_type_id: u64) {
        self.history.push(CollapseHistoryItem {
            position: *position,
            tile_type_id,
        });
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

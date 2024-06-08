use std::marker::PhantomData;

use rand::Rng;

use crate::gen::collapse::error::{CollapseError, CollapseErrorKind};
use crate::gen::collapse::grid::private::Sealed;
use crate::gen::collapse::queue::CollapseQueue;
use crate::gen::collapse::tile::CollapsibleTileData;
use crate::gen::collapse::{PropagateItem, Propagator};

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

    pub fn generate<R, Queue>(
        &mut self,
        mut grid: CollapsiblePatternGrid<P, Data>,
        rng: &mut R,
        positions: &[GridPosition],
        mut queue: Queue,
    ) -> Result<CollapsiblePatternGrid<P, Data>, CollapseError>
    where
        R: Rng,
        Queue: CollapseQueue,
    {
        use crate::gen::collapse::tile::private::Sealed;
        let mut propagator = Propagator::default();

        queue.populate_inner_grid(rng, &mut grid.pattern_grid, positions, &grid.option_data);

        for initial_propagate in grid._get_initial_propagate_items(positions) {
            propagator.push_propagate(initial_propagate);
        }

        CollapseError::from_result(
            propagator.propagate(&mut grid.pattern_grid, &grid.option_data, &mut queue, None),
            CollapseErrorKind::Init,
        )?;

        while let Some(collapse_position) = queue.get_next_position() {
            let mut to_collapse = grid
                .pattern_grid
                .get_mut_tile_at_position(&collapse_position)
                .unwrap();

            if to_collapse.as_ref().is_collapsed() {
                continue;
            }

            if !to_collapse.as_ref().has_compatible_options() {
                return Err(CollapseError::new(
                    collapse_position,
                    CollapseErrorKind::Collapse,
                ));
            }

            let Some(removed_options) = to_collapse.as_mut().collapse(rng, &grid.option_data)
            else {
                return Err(CollapseError::new(
                    collapse_position,
                    CollapseErrorKind::Collapse,
                ));
            };

            let collapsed_idx = to_collapse.as_ref().collapse_idx().unwrap();

            if let Some(subscriber) = self.subscriber.as_mut() {
                let pattern_id = grid.option_data.get_tile_type_id(&collapsed_idx).unwrap();
                let collapsed_id = grid
                    .patterns
                    .get_tile_data(&pattern_id)
                    .unwrap()
                    .tile_type_id();

                subscriber
                    .as_mut()
                    .on_collapse(&collapse_position, collapsed_id, pattern_id);
            }

            for removed_option in removed_options.into_iter() {
                propagator.push_propagate(PropagateItem::new(collapse_position, removed_option))
            }

            CollapseError::from_result(
                propagator.propagate(
                    &mut grid.pattern_grid,
                    &grid.option_data,
                    &mut queue,
                    Some(&collapse_position),
                ),
                CollapseErrorKind::Propagation,
            )?;
        }

        Ok(grid)
    }
}

pub trait Subscriber {
    fn on_collapse(&mut self, position: &GridPosition, tile_type_id: u64, pattern_id: u64);
}

pub struct DebugSubscriber;

impl Subscriber for DebugSubscriber {
    fn on_collapse(&mut self, position: &GridPosition, tile_type_id: u64, pattern_id: u64) {
        println!(
            "tile_type_id: {tile_type_id}, pattern_id: {pattern_id} on position: {position:?}"
        );
    }
}

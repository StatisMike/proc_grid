use std::marker::PhantomData;

use crate::gen::collapse::grid::private::Sealed;
use crate::gen::collapse::grid::CollapsibleGrid;
use crate::gen::collapse::{CollapsibleTileData, PropagateItem, Propagator};
use crate::tile::identifiable::collection::IdentTileCollection;
use crate::tile::identifiable::IdentifiableTileData;
use crate::tile::GridPosition;

use crate::gen::collapse::error::{CollapseError, CollapseErrorKind};
use crate::gen::collapse::queue::CollapseQueue;

use super::CollapsibleTileGrid;

use rand::Rng;

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
    pub fn with_subscriber(mut self, subscriber: Box<dyn Subscriber>) -> Self {
        self.subscriber = Some(subscriber);
        self
    }

    pub fn generate<R, Queue>(
        &mut self,
        grid: &mut CollapsibleTileGrid<Data>,
        rng: &mut R,
        positions: &[GridPosition],
        mut queue: Queue,
    ) -> Result<(), CollapseError>
    where
        R: Rng,
        Queue: CollapseQueue,
    {
        use crate::gen::collapse::tile::private::Sealed;
        let mut propagator = Propagator::default();

        grid.remove_uncollapsed();

        queue.populate_inner_grid(rng, &mut grid.grid, positions, &grid.option_data);

        for initial_propagate in grid._get_initial_propagate_items(positions) {
            propagator.push_propagate(initial_propagate);
        }

        CollapseError::from_result(
            propagator.propagate(&mut grid.grid, &grid.option_data, &mut queue, None),
            CollapseErrorKind::Init,
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
                propagator.propagate(
                    &mut grid.grid,
                    &grid.option_data,
                    &mut queue,
                    Some(&collapse_position),
                ),
                CollapseErrorKind::Propagation,
            )?;
        }

        Ok(())
    }
}

/// When applied to the struct allows injecting it into [`adjacency::Resolver`](Resolver) to react on each tile being collapsed.
pub trait Subscriber {
    fn on_collapse(&mut self, position: &GridPosition, tile_type_id: u64);
}

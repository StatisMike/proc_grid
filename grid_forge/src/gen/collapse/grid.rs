use std::collections::{HashMap, HashSet};

use crate::{
    map::{GridMap2D, GridSize},
    tile::{
        identifiable::{builders::IdentTileBuilder, IdentifiableTileData},
        GridPosition, GridTile, TileContainer,
    },
};

use super::{
    error::CollapsedGridError,
    option::PerOptionData,
    singular::{AdjacencyRules, CollapsibleTile, FrequencyHints},
    CollapsedTileData, CollapsibleTileData, PropagateItem,
};

/// [`GridMap2D`] containing data of [`CollapsedTileData`].
pub struct CollapsedGrid {
    grid: GridMap2D<CollapsedTileData>,
    tile_type_ids: HashSet<u64>,
}

impl CollapsedGrid {
    pub fn new(size: GridSize) -> Self {
        Self {
            grid: GridMap2D::new(size),
            tile_type_ids: HashSet::new(),
        }
    }

    pub fn insert_data(&mut self, position: &GridPosition, data: CollapsedTileData) -> bool {
        let tile_id = data.tile_type_id();
        if self.grid.insert_data(position, data) {
            self.tile_type_ids.insert(tile_id);
            true
        } else {
            false
        }
    }

    pub fn insert_tile(&mut self, tile: GridTile<CollapsedTileData>) -> bool {
        self.insert_data(&tile.grid_position(), tile.into_inner())
    }

    pub fn tile_type_ids(&self) -> impl Iterator<Item = &u64> {
        self.tile_type_ids.iter()
    }
}

impl AsRef<GridMap2D<CollapsedTileData>> for CollapsedGrid {
    fn as_ref(&self) -> &GridMap2D<CollapsedTileData> {
        &self.grid
    }
}

pub trait CollapsibleGrid<Tile: IdentifiableTileData>:
    Sized + private::Sealed<CollapsibleTile>
{
    fn new_empty(
        size: GridSize,
        frequencies: &FrequencyHints<Tile>,
        adjacencies: &AdjacencyRules<Tile>,
    ) -> Self;

    fn new_from_collapsed(
        collapsed: &CollapsedGrid,
        frequencies: &FrequencyHints<Tile>,
        adjacencies: &AdjacencyRules<Tile>,
    ) -> Result<Self, CollapsedGridError>;

    fn change(
        self,
        frequencies: &FrequencyHints<Tile>,
        adjacencies: &AdjacencyRules<Tile>,
    ) -> Result<Self, CollapsedGridError>;

    fn populate_from_collapsed(
        &mut self,
        collapsed: &CollapsedGrid,
    ) -> Result<(), CollapsedGridError>;

    fn retrieve_collapsed(&self) -> CollapsedGrid;

    fn retrieve_ident<OT: IdentifiableTileData, B: IdentTileBuilder<OT>>(
        &self,
        builder: &B,
    ) -> Result<GridMap2D<OT>, CollapsedGridError>;

    fn empty_positions(&self) -> Vec<GridPosition> {
        self._grid().get_all_empty_positions()
    }

    fn remove_uncollapsed(&mut self) {
        for t in self._grid_mut().iter_mut() {
            if let Some(d) = t {
                if d.is_collapsed() {
                    continue;
                }
                t.take();
            }
        }
    }
}

pub(crate) fn get_initial_propagate_items<T: CollapsibleTileData>(
    to_collapse: &[GridPosition],
    map: &GridMap2D<T>,
    option_data: &PerOptionData,
) -> Vec<PropagateItem> {
    let mut out = Vec::new();
    let mut cache = HashMap::new();
    let mut check_generated = HashSet::new();
    let check_provided: HashSet<_> = HashSet::from_iter(to_collapse.iter());

    for pos_to_collapse in to_collapse {
        for neighbour_tile in map.get_neighbours(pos_to_collapse) {
            if !neighbour_tile.as_ref().is_collapsed()
                || check_provided.contains(&neighbour_tile.grid_position())
                || check_generated.contains(&neighbour_tile.grid_position())
            {
                continue;
            }
            let check_pos = neighbour_tile.grid_position();
            check_generated.insert(check_pos);
            let collapsed_idx = neighbour_tile.as_ref().collapse_idx().unwrap();
            for opt_to_remove in cache.entry(collapsed_idx).or_insert_with(|| {
                (0..option_data.num_options())
                    .filter(|option_idx| option_idx != &collapsed_idx)
                    .collect::<Vec<usize>>()
            }) {
                out.push(PropagateItem::new(check_pos, *opt_to_remove))
            }
        }
    }
    out
}

pub(crate) mod private {
    use crate::{
        gen::collapse::{option::PerOptionData, CollapsibleTileData, PropagateItem},
        map::GridMap2D,
        tile::GridPosition,
    };

    pub trait Sealed<Tile: CollapsibleTileData> {
        #[doc(hidden)]
        fn _grid(&self) -> &GridMap2D<Tile>;

        #[doc(hidden)]
        fn _grid_mut(&mut self) -> &mut GridMap2D<Tile>;

        #[doc(hidden)]
        fn _option_data(&self) -> &PerOptionData;

        #[doc(hidden)]
        fn _get_initial_propagate_items(&self, to_collapse: &[GridPosition]) -> Vec<PropagateItem>;
    }
}

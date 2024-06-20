use std::collections::HashSet;

use crate::{
    map::{GridMap2D, GridSize},
    tile::{
        identifiable::{builders::IdentTileBuilder, IdentifiableTileData},
        GridPosition, GridTile, GridTileRef, TileContainer,
    },
};

use super::{error::CollapsibleGridError, CollapsedTileData, CollapsibleTileData};

/// [`GridMap2D`] containing data of [`CollapsedTileData`].
pub struct CollapsedGrid {
    grid: GridMap2D<CollapsedTileData>,
    tile_type_ids: HashSet<u64>,
}

impl CollapsedGrid {
    /// Creates new [`CollapsedGrid`] with the given size.
    pub fn new(size: GridSize) -> Self {
        Self {
            grid: GridMap2D::new(size),
            tile_type_ids: HashSet::new(),
        }
    }

    /// Inserts [`CollapsedTileData`] into the specified position in the internal grid.
    pub fn insert_data(&mut self, position: &GridPosition, data: CollapsedTileData) -> bool {
        let tile_id = data.tile_type_id();
        if self.grid.insert_data(position, data) {
            self.tile_type_ids.insert(tile_id);
            true
        } else {
            false
        }
    }

    /// Inserts [`GridTile`] of [`CollapsedTileData`] into the internal grid.
    pub fn insert_tile(&mut self, tile: GridTile<CollapsedTileData>) -> bool {
        self.insert_data(&tile.grid_position(), tile.into_inner())
    }

    /// Returns iterator over all `tile_type_id`s of the collapsed tiles in the grid.
    pub fn tile_type_ids(&self) -> impl Iterator<Item = &u64> {
        self.tile_type_ids.iter()
    }
}

impl AsRef<GridMap2D<CollapsedTileData>> for CollapsedGrid {
    fn as_ref(&self) -> &GridMap2D<CollapsedTileData> {
        &self.grid
    }
}

/// Trait shared by a structs holding a grid of [`CollapsibleTileData`], useable by dedicated resolvers to collapse
/// the grid.
pub trait CollapsibleGrid<IT: IdentifiableTileData, CT: CollapsibleTileData>:
    Sized + private::Sealed<CT>
{
    /// Retrieves the collapsed tiles in internal grid as a [`CollapsedGrid`].
    fn retrieve_collapsed(&self) -> CollapsedGrid;

    /// Retrieves the collapsed tiles in internal grid as a [`GridMap2D`] of [`IdentifiableTileData`].
    fn retrieve_ident<OT: IdentifiableTileData, B: IdentTileBuilder<OT>>(
        &self,
        builder: &B,
    ) -> Result<GridMap2D<OT>, CollapsibleGridError>;

    /// Returns all empty positions in the internal grid.
    fn empty_positions(&self) -> Vec<GridPosition> {
        self._grid().get_all_empty_positions()
    }

    /// Returns all possitions in the internal grid holds collapsed or uncollapsed tiles are either collapsed.
    fn retrieve_positions(&self, collapsed: bool) -> Vec<GridPosition> {
        let func = if collapsed {
            |t: &GridTileRef<CT>| t.as_ref().is_collapsed()
        } else {
            |t: &GridTileRef<CT>| !t.as_ref().is_collapsed()
        };
        self._grid()
            .iter_tiles()
            .filter_map(|t| {
                if func(&t) {
                    Some(t.grid_position())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Removes all uncollapsed tiles from the internal grid.
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

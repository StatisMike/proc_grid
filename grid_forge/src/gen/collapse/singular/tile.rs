use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

use rand::Rng;

use crate::gen::collapse::error::CollapsedGridError;
use crate::gen::collapse::grid::CollapsibleGrid;
use crate::gen::collapse::option::{PerOptionData, WaysToBeOption};
use crate::gen::collapse::{self, tile::*, CollapsedGrid, PropagateItem};
use crate::map::{GridMap2D, GridSize};
use crate::tile::identifiable::builders::IdentTileBuilder;
use crate::tile::identifiable::collection::IdentTileCollection;
use crate::tile::identifiable::IdentifiableTileData;
use crate::tile::{GridPosition, GridTile, TileContainer, TileData};

use super::{AdjacencyRules, FrequencyHints};

/// Tile with options that can be collapsed into one of them.
#[derive(Clone, Debug)]
pub struct CollapsibleTile {
    collapsed_option: Option<usize>,
    num_possible_options: usize,
    ways_to_be_option: WaysToBeOption,
    weight_sum: u32,
    weight_log_sum: f32,
    entrophy_noise: f32,
}

impl TileData for CollapsibleTile {}

impl crate::gen::collapse::tile::private::Sealed for CollapsibleTile {
    fn remove_option(&mut self, weights: (u32, f32)) {
        self.num_possible_options -= 1;
        self.weight_sum -= weights.0;
        self.weight_log_sum -= weights.1;
    }

    fn new_uncollapsed_tile(
        position: GridPosition,
        num_possible_options: usize,
        ways_to_be_option: WaysToBeOption,
        weight_sum: u32,
        weight_log_sum: f32,
        entrophy_noise: f32,
    ) -> GridTile<Self>
    where
        Self: TileData,
    {
        GridTile::new(
            position,
            Self {
                collapsed_option: None,
                num_possible_options,
                ways_to_be_option,
                weight_sum,
                weight_log_sum,
                entrophy_noise,
            },
        )
    }

    fn ways_to_be_option(&self) -> &WaysToBeOption {
        &self.ways_to_be_option
    }

    fn mut_ways_to_be_option(&mut self) -> &mut WaysToBeOption {
        &mut self.ways_to_be_option
    }

    fn num_possible_options(&self) -> usize {
        self.num_possible_options
    }

    fn collapse<R: Rng>(
        &mut self,
        rng: &mut R,
        options_data: &PerOptionData,
    ) -> Option<Vec<usize>> {
        assert!(self.weight_sum > 0);
        let random = rng.gen_range(0..self.weight_sum);
        let mut current_sum = 0;
        let mut chosen = None;
        let mut out = Vec::new();
        for option_idx in self.ways_to_be_option().iter_possible() {
            current_sum += options_data.get_weights(option_idx).0;
            if chosen.is_some() || random > current_sum {
                out.push(option_idx);
                continue;
            }
            chosen = Some(option_idx);
        }
        assert!(chosen.is_some(), "option should always be chosen!");
        self.collapsed_option = chosen;
        self.num_possible_options = 0;
        self.weight_sum = 0;
        self.weight_log_sum = 0.;
        Some(out)
    }

    fn mark_collapsed(&mut self, collapsed_idx: usize) {
        self.collapsed_option = Some(collapsed_idx);
        self.num_possible_options = 0;
        self.weight_sum = 0;
        self.weight_log_sum = 0.;
    }

    fn weight_sum(&self) -> u32 {
        self.weight_sum
    }
}

impl CollapsibleTileData for CollapsibleTile {
    fn collapse_idx(&self) -> Option<usize> {
        self.collapsed_option
    }

    fn calc_entrophy(&self) -> f32 {
        Self::calc_entrophy_ext(self.weight_sum, self.weight_log_sum) + self.entrophy_noise
    }

    fn num_compatible_options(&self) -> usize {
        self.num_possible_options
    }

    fn new_collapsed_data(option_idx: usize) -> Self {
        Self {
            collapsed_option: Some(option_idx),
            num_possible_options: 0,
            ways_to_be_option: WaysToBeOption::default(),
            weight_sum: 0,
            weight_log_sum: 0.,
            entrophy_noise: 0.,
        }
    }
}

pub struct CollapsibleTileGrid<Tile: IdentifiableTileData> {
    pub(crate) grid: GridMap2D<CollapsibleTile>,
    pub(crate) option_data: PerOptionData,
    tile_type: PhantomData<Tile>,
}

impl<Tile: IdentifiableTileData> CollapsibleTileGrid<Tile> {
    pub fn new_empty(
        size: GridSize,
        frequencies: &FrequencyHints<Tile>,
        adjacencies: &AdjacencyRules<Tile>,
    ) -> Self {
        let mut option_data = PerOptionData::default();
        option_data.populate(&frequencies.get_all_weights_cloned(), adjacencies.inner());

        Self {
            grid: GridMap2D::new(size),
            option_data,
            tile_type: PhantomData,
        }
    }

    pub fn new_from_collapsed(
        collapsed: &CollapsedGrid,
        frequencies: &FrequencyHints<Tile>,
        adjacencies: &AdjacencyRules<Tile>,
    ) -> Result<Self, CollapsedGridError> {
        let mut option_data = PerOptionData::default();
        option_data.populate(&frequencies.get_all_weights_cloned(), adjacencies.inner());

        let missing_ids = collapsed
            .tile_type_ids()
            .filter(|id| !option_data.inner().keys().any(|k| k == *id))
            .copied()
            .collect::<Vec<_>>();

        if !missing_ids.is_empty() {
            return Err(CollapsedGridError::new_missing(missing_ids));
        }

        let mut grid = GridMap2D::new(*collapsed.as_ref().size());

        for tile in collapsed.as_ref().iter_tiles() {
            grid.insert_data(
                &tile.grid_position(),
                CollapsibleTile::new_collapsed_data(
                    *option_data
                        .get_tile_data(&tile.as_ref().tile_type_id())
                        .expect("cannot get `option_idx`"),
                ),
            );
        }

        Ok(Self {
            grid,
            option_data,
            tile_type: PhantomData,
        })
    }

    pub fn change(
        self,
        frequencies: &FrequencyHints<Tile>,
        adjacencies: &AdjacencyRules<Tile>,
    ) -> Result<Self, CollapsedGridError> {
        let collapsed = self.retrieve_collapsed();

        Self::new_from_collapsed(&collapsed, frequencies, adjacencies)
    }

    pub fn populate_from_collapsed(
        &mut self,
        collapsed: &CollapsedGrid,
    ) -> Result<(), CollapsedGridError> {
        if !self
            .grid
            .size
            .is_contained_within(collapsed.as_ref().size())
        {
            return Err(CollapsedGridError::new_wrong_size(
                collapsed.as_ref().size,
                self.grid.size,
            ));
        }

        let missing_ids = collapsed
            .tile_type_ids()
            .filter(|id| !self.option_data.inner().keys().any(|k| k == *id))
            .copied()
            .collect::<Vec<_>>();

        if !missing_ids.is_empty() {
            return Err(CollapsedGridError::new_missing(missing_ids));
        }

        for tile in collapsed.as_ref().iter_tiles() {
            self.grid.insert_data(
                &tile.grid_position(),
                CollapsibleTile::new_collapsed_data(
                    *self
                        .option_data
                        .get_tile_data(&tile.as_ref().tile_type_id())
                        .expect("cannot get `option_idx`"),
                ),
            );
        }

        Ok(())
    }
}

impl<Tile: IdentifiableTileData> CollapsibleGrid<Tile, CollapsibleTile>
    for CollapsibleTileGrid<Tile>
{
    fn retrieve_collapsed(&self) -> CollapsedGrid {
        let mut out = CollapsedGrid::new(*self.grid.size());

        for tile in self.grid.iter_tiles() {
            if !tile.as_ref().is_collapsed() {
                continue;
            }
            out.insert_data(
                &tile.grid_position(),
                CollapsedTileData::new(
                    self.option_data
                        .get_tile_type_id(
                            &tile
                                .as_ref()
                                .collapse_idx()
                                .expect("cannot get collapse idx"),
                        )
                        .expect("cannot get option id for collapse idx"),
                ),
            );
        }

        out
    }

    fn retrieve_ident<T: IdentifiableTileData, B: IdentTileBuilder<T>>(
        &self,
        builder: &B,
    ) -> Result<GridMap2D<T>, CollapsedGridError> {
        let mut out = GridMap2D::new(*self.grid.size());

        for tile in self.grid.iter_tiles() {
            if !tile.as_ref().is_collapsed() {
                continue;
            }
            out.insert_tile(
                builder.build_tile_unchecked(
                    tile.grid_position(),
                    self.option_data
                        .get_tile_type_id(
                            &tile
                                .as_ref()
                                .collapse_idx()
                                .expect("cannot get collapse idx"),
                        )
                        .expect("cannot get option id for collapse idx"),
                ),
            );
        }

        Ok(out)
    }
}

impl<Tile: IdentifiableTileData> collapse::grid::private::Sealed<CollapsibleTile>
    for CollapsibleTileGrid<Tile>
{
    fn _option_data(&self) -> &PerOptionData {
        &self.option_data
    }

    fn _grid_mut(&mut self) -> &mut GridMap2D<CollapsibleTile> {
        &mut self.grid
    }

    fn _grid(&self) -> &GridMap2D<CollapsibleTile> {
        &self.grid
    }

    fn _get_initial_propagate_items(&self, to_collapse: &[GridPosition]) -> Vec<PropagateItem> {
        let mut out = Vec::new();
        let mut cache = HashMap::new();
        let mut check_generated = HashSet::new();
        let check_provided: HashSet<_> = HashSet::from_iter(to_collapse.iter());

        for pos_to_collapse in to_collapse {
            for neighbour_tile in self.grid.get_neighbours(pos_to_collapse) {
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
                    (0..self.option_data.num_options())
                        .filter(|option_idx| option_idx != &collapsed_idx)
                        .collect::<Vec<usize>>()
                }) {
                    out.push(PropagateItem::new(check_pos, *opt_to_remove))
                }
            }
        }
        out
    }
}

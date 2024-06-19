use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

use rand::Rng;

use crate::gen::collapse::error::CollapsibleGridError;
use crate::gen::collapse::grid::CollapsibleGrid;
use crate::gen::collapse::option::{PerOptionData, WaysToBeOption};
use crate::gen::collapse::{self, tile::*, CollapsedGrid, PropagateItem};
use crate::map::{GridMap2D, GridSize};
use crate::tile::identifiable::builders::IdentTileBuilder;
use crate::tile::identifiable::collection::IdentTileCollection;
use crate::tile::identifiable::IdentifiableTileData;
use crate::tile::{GridPosition, GridTile, TileContainer, TileData};

use super::{AdjacencyRules, FrequencyHints};

/// Tile with options that can be collapsed into one of them. Mostly used within the [`CollapsibleTileGrid`].
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

/// Collapsible grid compatible with [`singular::Resolver`](super::Resolver).
/// 
/// It stores the data of the tiles in the internal grid of [`CollapsibleTile`]. Holds information about the rules for
/// the generation of the tiles and the weights of the options, provide options to populate the grid with some collapsed
/// tiles before the generation process and retrieve the collapsed tiles after the generation ends.
/// 
/// ## Example
/// ```
/// use grid_forge::gen::collapse::*;
/// use grid_forge::tile::GridPosition;
/// use grid_forge::map::GridSize;
/// use grid_forge::tile::identifiable::*;
/// use grid_forge::tile::identifiable::builders::*;
/// #
/// # // setup to prepopulate the rules.
/// # use grid_forge::tile::*;
/// # use grid_forge::map::GridDir;
/// # use grid_forge::tile::identifiable::*;
/// # use grid_forge::tile::identifiable::builders::*;
/// # let first_tile = GridTile::new(GridPosition::new_xy(0,0), BasicIdentTileData::tile_new(0));
/// # let second_tile = GridTile::new(GridPosition::new_xy(1,0), BasicIdentTileData::tile_new(1));
/// # let mut adjacency_rules = singular::AdjacencyRules::<BasicIdentTileData>::default(); 
/// # adjacency_rules.add_adjacency(&first_tile, &second_tile, GridDir::UP);
/// # adjacency_rules.add_adjacency(&second_tile, &first_tile, GridDir::LEFT);
/// # let mut frequency_hints = singular::FrequencyHints::<BasicIdentTileData>::default();
/// # frequency_hints.set_weight_for_tile(&first_tile, 1);
/// # frequency_hints.set_weight_for_tile(&second_tile, 2);
/// 
/// // Create new empty grid.
/// let mut collapsible_grid = singular::CollapsibleTileGrid::new_empty(GridSize::new_xy(10, 10), &frequency_hints, &adjacency_rules);
/// 
/// // We can prepopulate existing collapsible grid with some collapsed tiles using `CollapsedGrid`.
/// let mut collapsed_grid = CollapsedGrid::new(GridSize::new_xy(10, 10));
/// collapsed_grid.insert_data(&GridPosition::new_xy(0, 0), CollapsedTileData::new(0));
/// collapsed_grid.insert_data(&GridPosition::new_xy(1, 0), CollapsedTileData::new(1));
/// 
/// collapsible_grid.populate_from_collapsed(&collapsed_grid).unwrap();
/// 
/// // The collapsible grid can be created directly from a `CollapsedGrid`.
/// let mut collapsible_grid = singular::CollapsibleTileGrid::new_from_collapsed(&collapsed_grid, &frequency_hints, &adjacency_rules).unwrap();
/// 
/// // If there is a need to change the rules after the grid is created, it can be done using `change` method.
/// # let mut new_frequency_hints = singular::FrequencyHints::<BasicIdentTileData>::default();
/// # new_frequency_hints.set_weight_for_tile(&first_tile, 2);
/// # new_frequency_hints.set_weight_for_tile(&second_tile, 1);
/// collapsible_grid = collapsible_grid.change(&new_frequency_hints, &adjacency_rules).unwrap();
/// 
/// // The grid can be retrieved as a `CollapsedGrid`.
/// let collapsed_grid = collapsible_grid.retrieve_collapsed();
/// 
/// // The grid can be retrieved as a `GridMap2D` using compatible `IdentTileBuilder`.
/// let ident_grid = collapsible_grid.retrieve_ident(&IdentTileTraitBuilder::<BasicIdentTileData>::default());
/// ```
pub struct CollapsibleTileGrid<Tile: IdentifiableTileData> {
    pub(crate) grid: GridMap2D<CollapsibleTile>,
    pub(crate) option_data: PerOptionData,
    tile_type: PhantomData<Tile>,
}

impl<Tile: IdentifiableTileData> CollapsibleTileGrid<Tile> {

    /// Creates a new empty grid with given [`GridSize`], preparing the rules for the generation of the tiles and the weights of the options.
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

    /// Creates a new grid using the [`CollapsedGrid`] as a source grid. Created grid will have the same size as the 
    /// source grid, and will be populated with existing collapsed tiles.
    /// 
    /// Method can return an error if the collapsed grid contains tiles with `tile_type_id`s that are not present in the
    /// provided frequency hints and adjacency rules.
    pub fn new_from_collapsed(
        collapsed: &CollapsedGrid,
        frequencies: &FrequencyHints<Tile>,
        adjacencies: &AdjacencyRules<Tile>,
    ) -> Result<Self, CollapsibleGridError> {
        let mut option_data = PerOptionData::default();
        option_data.populate(&frequencies.get_all_weights_cloned(), adjacencies.inner());

        let missing_ids = collapsed
            .tile_type_ids()
            .filter(|id| !option_data.inner().keys().any(|k| k == *id))
            .copied()
            .collect::<Vec<_>>();

        if !missing_ids.is_empty() {
            return Err(CollapsibleGridError::new_missing(missing_ids));
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

    /// Changes the rules for the generation of the tiles and the weights of the options.
    /// 
    /// Method can return an error if the inner collapsible grid contains tiles with `tile_type_id`s that are not present in the
    /// provided frequency hints and adjacency rules.
    pub fn change(
        self,
        frequencies: &FrequencyHints<Tile>,
        adjacencies: &AdjacencyRules<Tile>,
    ) -> Result<Self, CollapsibleGridError> {
        let collapsed = self.retrieve_collapsed();

        Self::new_from_collapsed(&collapsed, frequencies, adjacencies)
    }

    /// Populates the grid with all collapsed tiles from the provided [`CollapsedGrid`].
    /// 
    /// Method can return an error if the provided grid contains tiles with `tile_type_id`s that are not present in the
    /// provided frequency hints and adjacency rules or the provided grid size is greater than the size of inner
    /// collapsible grid.
    pub fn populate_from_collapsed(
        &mut self,
        collapsed: &CollapsedGrid,
    ) -> Result<(), CollapsibleGridError> {
        if !self
            .grid
            .size
            .is_contained_within(collapsed.as_ref().size())
        {
            return Err(CollapsibleGridError::new_wrong_size(
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
            return Err(CollapsibleGridError::new_missing(missing_ids));
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
    ) -> Result<GridMap2D<T>, CollapsibleGridError> {
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

use crate::tile::identifiable::builders::ConstructableViaIdentifierTile;
use crate::tile::identifiable::IdentifiableTileData;
use crate::tile::TileData;

/// Simple [`TileData`] containing only the `tile_type_id`.
///
/// Identical in most cases to [`BasicIdentTileData`](crate::tile::identifiable::BasicIdentTileData), but used consistently within the
/// collapse algorithms - both as input for some initial constraints for the generation process, and as an collapse process output.
pub struct CollapsedTileData {
    tile_type_id: u64,
}

impl TileData for CollapsedTileData {}

impl IdentifiableTileData for CollapsedTileData {
    fn tile_type_id(&self) -> u64 {
        self.tile_type_id
    }
}

impl ConstructableViaIdentifierTile for CollapsedTileData {
    fn tile_new(tile_type_id: u64) -> Self {
        Self::new(tile_type_id)
    }
}

impl CollapsedTileData {
    #[inline]
    pub fn new(tile_type_id: u64) -> Self {
        Self { tile_type_id }
    }
}

/// Trait shared by [`TileData`] used within collapsible generative algorithms.
pub trait CollapsibleTileData: TileData + private::Sealed {
    /// Returns number of possible options for the tile.
    fn num_compatible_options(&self) -> usize;

    /// Checks if the tile has any possible options.
    fn has_compatible_options(&self) -> bool {
        self.num_possible_options() > 0
    }

    /// Checks if the tile is collapsed.
    fn is_collapsed(&self) -> bool {
        self.collapse_idx().is_some()
    }

    /// Returns the index of the collapsed option.
    fn collapse_idx(&self) -> Option<usize>;

    /// Create new collapsed tile data.
    fn new_collapsed_data(option_idx: usize) -> Self;

    /// Calculate entrophy.
    fn calc_entrophy(&self) -> f32;

    /// Associated function to calculate entrophy.
    #[inline]
    fn calc_entrophy_ext(weight_sum: u32, weight_log_sum: f32) -> f32 {
        (weight_sum as f32).log2() - weight_log_sum / (weight_sum as f32)
    }
}

pub(crate) mod private {
    use std::collections::HashSet;

    use rand::{
        distributions::{Distribution, Uniform},
        Rng,
    };

    use crate::{
        gen::collapse::option::{PerOptionData, WaysToBeOption},
        map::{GridDir, GridMap2D},
        tile::{self, GridPosition, GridTile},
    };

    use super::CollapsibleTileData;

    /// Sealed trait for the [`CollapsibleTileData`] trait. It contains most of the shared logic for its implementors,
    /// which should be kept private.
    pub trait Sealed: tile::TileData {
        /// Creates new uncollapsed tile.
        fn new_uncollapsed_tile(
            position: GridPosition,
            num_options: usize,
            ways_to_be_option: WaysToBeOption,
            weight_sum: u32,
            weight_log_sum: f32,
            entrophy_noise: f32,
        ) -> GridTile<Self>;

        /// Creates vector of uncollapsed tiles with entrophy noise.
        fn new_from_frequency_with_entrophy<R: Rng>(
            rng: &mut R,
            positions: &[GridPosition],
            options_data: &PerOptionData,
        ) -> Vec<GridTile<Self>> {
            let rng_range = Self::entrophy_uniform();
            let ways_to_be_option = options_data.get_ways_to_become_option();

            let (weight_sum, weight_log_sum) = ways_to_be_option
                .iter_possible()
                .map(|option_idx| options_data.get_weights(option_idx))
                .fold(
                    (0u32, 0f32),
                    |(sum_weight, sum_weight_log), (weight, weight_log)| {
                        (sum_weight + weight, sum_weight_log + weight_log)
                    },
                );

            positions
                .iter()
                .map(|position| {
                    Self::new_uncollapsed_tile(
                        *position,
                        options_data.num_possible_options(),
                        ways_to_be_option.clone(),
                        weight_sum,
                        weight_log_sum,
                        rng_range.sample(rng),
                    )
                })
                .collect::<Vec<_>>()
        }

        /// Creates vector of uncollapsed tiles.
        fn new_from_frequency(
            positions: &[GridPosition],
            options_data: &PerOptionData,
        ) -> Vec<GridTile<Self>> {
            let ways_to_be_option = options_data.get_ways_to_become_option();

            let (weight_sum, weight_log_sum) = ways_to_be_option
                .iter_possible()
                .map(|option_idx| options_data.get_weights(option_idx))
                .fold(
                    (0u32, 0f32),
                    |(sum_weight, sum_weight_log), (weight, weight_log)| {
                        (sum_weight + weight, sum_weight_log + weight_log)
                    },
                );

            positions
                .iter()
                .map(|pos| {
                    Self::new_uncollapsed_tile(
                        *pos,
                        options_data.num_possible_options(),
                        ways_to_be_option.clone(),
                        weight_sum,
                        weight_log_sum,
                        0.0,
                    )
                })
                .collect::<Vec<_>>()
        }

        fn num_possible_options(&self) -> usize;

        fn ways_to_be_option(&self) -> &WaysToBeOption;

        fn mut_ways_to_be_option(&mut self) -> &mut WaysToBeOption;

        /// Removes single option from tile.
        fn remove_option(&mut self, weights: (u32, f32));

        /// Range of uniformly distributed data for entrophy noise.
        fn entrophy_uniform() -> Uniform<f32> {
            Uniform::<f32>::new(0., 0.00001)
        }

        fn collapse<R: Rng>(
            &mut self,
            rng: &mut R,
            options_data: &PerOptionData,
        ) -> Option<Vec<usize>>;

        fn mark_collapsed(&mut self, collapsed_idx: usize);

        fn weight_sum(&self) -> u32;

        /// Collapses tile into one of possible options, returning the vector of the removed options.
        fn collapse_gather_removed<R: Rng>(
            &mut self,
            rng: &mut R,
            options_data: &PerOptionData,
        ) -> Vec<usize> {
            assert!(
                self.weight_sum() > 0,
                "weight sum should be positive when collapsing!"
            );
            let random = rng.gen_range(0..self.weight_sum());
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
            self.mark_collapsed(chosen.expect("options should always be chosen"));
            out
        }

        /// Collapses tiles into one of possible options.
        fn collapse_basic<R: Rng>(&mut self, rng: &mut R, options_data: &PerOptionData) {
            assert!(
                self.weight_sum() > 0,
                "weight sum should be positive when collapsing!"
            );
            let random = rng.gen_range(0..self.weight_sum());
            let mut current_sum = 0;
            let mut chosen = None;
            for option_idx in self.ways_to_be_option().iter_possible() {
                current_sum += options_data.get_weights(option_idx).0;
                if chosen.is_some() || random > current_sum {
                    continue;
                }
                chosen = Some(option_idx);
            }
            self.mark_collapsed(chosen.expect("options should always be chosen"));
        }

        /// Removes options from tile neighbours after its collapse.
        fn purge_options_for_neighbours(
            grid: &mut GridMap2D<Self>,
            collapsed_option: usize,
            collapsed_position: &GridPosition,
            option_data: &PerOptionData,
        ) where
            Self: CollapsibleTileData,
        {
            for direction in GridDir::ALL_2D {
                if let Some(mut tile) = grid.get_mut_neighbour_at(collapsed_position, direction) {
                    if tile.as_ref().is_collapsed() {
                        continue;
                    }

                    let enabled =
                        option_data.get_all_enabled_in_direction(collapsed_option, *direction);
                    for possible_option in tile
                        .as_ref()
                        .ways_to_be_option()
                        .iter_possible()
                        .collect::<Vec<_>>()
                    {
                        if !enabled.contains(&possible_option)
                            && tile
                                .as_mut()
                                .mut_ways_to_be_option()
                                .purge_option(possible_option)
                        {
                            let weights = option_data.get_weights(possible_option);
                            tile.as_mut().remove_option(weights);
                        }
                    }
                }
            }
        }

        /// Removes options from tile based of possible options for its neighbours.
        fn purge_incompatible_options(
            grid: &mut GridMap2D<Self>,
            position: &GridPosition,
            option_data: &PerOptionData,
        ) -> bool
        where
            Self: CollapsibleTileData,
        {
            let num_options = option_data.num_options();
            let mut possible_options = Vec::with_capacity(num_options);
            possible_options.resize(num_options, true);

            for direction in GridDir::ALL_2D {
                if let Some(tile) = grid.get_neighbour_at(position, direction) {
                    if let Some(collapsed_idx) = tile.as_ref().collapse_idx() {
                        let enabled = option_data
                            .get_all_enabled_in_direction(collapsed_idx, direction.opposite());
                        for (option_idx, state) in possible_options.iter_mut().enumerate() {
                            if *state && !enabled.contains(&option_idx) {
                                *state = false;
                            }
                        }
                    } else if tile.as_ref().num_compatible_options()
                        < option_data.num_possible_options()
                    {
                        let mut possible_in_any: HashSet<usize> = HashSet::new();
                        for neigbour_idx in tile.as_ref().ways_to_be_option().iter_possible() {
                            possible_in_any.extend(
                                option_data
                                    .get_all_enabled_in_direction(
                                        neigbour_idx,
                                        direction.opposite(),
                                    )
                                    .iter(),
                            );
                        }
                        for (option_idx, state) in possible_options.iter_mut().enumerate() {
                            if *state && !possible_in_any.contains(&option_idx) {
                                *state = false;
                            }
                        }
                    }
                }
            }

            if !possible_options.iter().any(|state| *state) {
                return false;
            }

            let mut tile = grid.get_mut_tile_at_position(position).unwrap();
            for (possible, (option_idx, weights)) in
                possible_options.iter().zip(option_data.iter_weights())
            {
                if !possible
                    && tile
                        .as_mut()
                        .mut_ways_to_be_option()
                        .purge_option(option_idx)
                {
                    tile.as_mut().remove_option(*weights);
                }
            }
            true
        }
    }
}

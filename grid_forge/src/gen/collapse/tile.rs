use std::collections::{BTreeMap, HashMap};

use rand::distributions::{Distribution, Uniform};
use rand::Rng;

use crate::map::GridDir;
use crate::tile::identifiable::IdentifiableTileData;
use crate::tile::{GridPosition, GridTile, TileData};

use super::{AdjacencyTable, DirectionTable};

pub struct CollapsedTileData {
    tile_type_id: u64,
}

impl TileData for CollapsedTileData {}

impl IdentifiableTileData for CollapsedTileData {
    fn tile_type_id(&self) -> u64 {
        self.tile_type_id
    }
}

/// Trait shared by [`TileData`] used within collapsible generative algorithms.
pub trait CollapsibleTileData: IdentifiableTileData + private::Sealed {
    fn have_options(&self) -> bool;

    fn remove_option(&mut self, tile_id: u64) -> bool;

    fn is_collapsed(&self) -> bool {
        self.collapse_id().is_some()
    }

    fn collapse_id(&self) -> Option<u64>;

    /// Create new collapsed tile.
    fn new_collapsed_tile(position: GridPosition, tile_id: u64) -> GridTile<Self>;

    /// Calculate entrophy.
    fn calc_entrophy(&self) -> f32;

    /// Associate function to calculate entrophy.
    fn calc_entrophy_ext(weight_sum: u32, weight_log_sum: f32) -> f32 {
        (weight_sum as f32).log2() - weight_log_sum / (weight_sum as f32)
    }

    // fn decrement_ways_to_be_option(
    //     &mut self,

    // ) ->
}

#[derive(Clone)]
pub struct WaysToBeOption {
    table: HashMap<u64, DirectionTable<usize>>,
}

impl WaysToBeOption {
    pub(crate) fn empty() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    pub(crate) fn new(adjacencies: &AdjacencyTable) -> Self {
        let mut table = HashMap::new();
        for (option_id, adjacencts_for_option) in adjacencies.inner.iter() {
            let mut ways_for_option_by_dir = DirectionTable::default();

            for direction in GridDir::ALL_2D {
                ways_for_option_by_dir[*direction] = adjacencts_for_option[*direction].len();
            }
            table.insert(*option_id, ways_for_option_by_dir);
        }
        Self { table }
    }

    /// Decrement
    pub(crate) fn try_decrement(
        &mut self,
        option_id: u64,
        direction: GridDir,
    ) -> WaysToBeOptionOutcome {
        if let Some(num_ways_by_dir) = self.table.get_mut(&option_id) {
            let mut num_ways = num_ways_by_dir[direction];
            if num_ways == 0 {
                return WaysToBeOptionOutcome::NoChange;
            }
            num_ways -= 1;
            if num_ways > 0 {
                return WaysToBeOptionOutcome::Decremented;
            }
            return WaysToBeOptionOutcome::Eliminated;
        }
        WaysToBeOptionOutcome::NoChange
    }
}

pub enum WaysToBeOptionOutcome {
    NoChange,
    Decremented,
    Eliminated,
}

pub(crate) mod private {
    use std::collections::BTreeMap;

    use rand::{
        distributions::{Distribution, Uniform},
        Rng,
    };

    use crate::{
        gen::collapse::AdjacencyTable,
        map::GridDir,
        tile::{self, GridPosition, GridTile},
    };

    use super::{WaysToBeOption, WaysToBeOptionOutcome};

    pub trait Sealed {
        fn new_uncollapsed_tile(
            position: GridPosition,
            options_with_weights: BTreeMap<u64, u32>,
            ways_to_be_option: WaysToBeOption,
            weight_sum: u32,
            weight_log_sum: f32,
            entrophy_noise: f32,
        ) -> GridTile<Self>
        where
            Self: tile::TileData;

        fn new_from_frequency_with_entrophy<R: Rng>(
            rng: &mut R,
            positions: &[GridPosition],
            adjacency_table: &AdjacencyTable,
            options_with_weights: BTreeMap<u64, u32>,
        ) -> Vec<GridTile<Self>>
        where
            Self: tile::TileData,
        {
            let rng_range = Self::entrophy_uniform();
            let weight_sum: u32 = options_with_weights.values().sum();
            let weight_log_sum = options_with_weights
                .values()
                .map(|v| (*v as f32) * (*v as f32).log2())
                .sum::<f32>();

            let ways_to_be_option = WaysToBeOption::new(adjacency_table);

            positions
                .iter()
                .map(|position| {
                    Self::new_uncollapsed_tile(
                        *position,
                        options_with_weights.clone(),
                        ways_to_be_option.clone(),
                        weight_sum,
                        weight_log_sum,
                        rng_range.sample(rng),
                    )
                })
                .collect::<Vec<_>>()
        }

        fn new_from_frequency(
            positions: &[GridPosition],
            adjacency_table: &AdjacencyTable,
            options_with_weights: BTreeMap<u64, u32>,
        ) -> Vec<GridTile<Self>>
        where
            Self: tile::TileData,
        {
            let weight_sum: u32 = options_with_weights.values().sum();
            let weight_log_sum = options_with_weights
                .values()
                .map(|v| (*v as f32) * (*v as f32).log2())
                .sum::<f32>();

            let ways_to_be_option = WaysToBeOption::new(adjacency_table);

            positions
                .iter()
                .map(|pos| {
                    Self::new_uncollapsed_tile(
                        *pos,
                        options_with_weights.clone(),
                        ways_to_be_option.clone(),
                        weight_sum,
                        weight_log_sum,
                        0.0,
                    )
                })
                .collect::<Vec<_>>()
        }

        fn ways_to_be_option(&mut self) -> &mut WaysToBeOption;

        fn decrement_ways_to_be_option(
            &mut self,
            option_id: u64,
            direction: GridDir,
        ) -> WaysToBeOptionOutcome {
            self.ways_to_be_option().try_decrement(option_id, direction)
        }

        fn entrophy_uniform() -> Uniform<f32> {
            Uniform::<f32>::new(0., 0.00001)
        }
    }
}

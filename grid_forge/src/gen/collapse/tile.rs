use std::collections::BTreeMap;

use rand::distributions::{Distribution, Uniform};
use rand::Rng;

use crate::tile::identifiable::IdentifiableTileData;
use crate::tile::{GridPosition, GridTile, TileData};

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
pub trait CollapsibleData: IdentifiableTileData {
    fn have_options(&self) -> bool;

    fn remove_option(&mut self, tile_id: u64) -> bool;

    fn is_collapsed(&self) -> bool {
        self.collapse_id().is_some()
    }

    fn collapse_id(&self) -> Option<u64>;

    fn new_uncollapsed_tile(
        position: GridPosition,
        options_with_weights: BTreeMap<u64, u32>,
        weight_sum: u32,
        weight_log_sum: f32,
        entrophy_noise: f32,
    ) -> GridTile<Self>;

    /// Create new collapsed tile.
    fn new_collapsed_tile(position: GridPosition, tile_id: u64) -> GridTile<Self>;

    /// Vector constructor where collapsible tiles need entrophy noise.
    fn new_from_frequency_with_entrophy<R: Rng>(
        rng: &mut R,
        positions: &[GridPosition],
        options_with_weights: BTreeMap<u64, u32>,
    ) -> Vec<GridTile<Self>> {
        let rng_range = Self::entrophy_uniform();
        let weight_sum: u32 = options_with_weights.values().sum();
        let weight_log_sum = options_with_weights
            .values()
            .map(|v| (*v as f32) * (*v as f32).log2())
            .sum::<f32>();

        positions
            .iter()
            .map(|position| {
                Self::new_uncollapsed_tile(
                    *position,
                    options_with_weights.clone(),
                    weight_sum,
                    weight_log_sum,
                    rng_range.sample(rng),
                )
            })
            .collect::<Vec<_>>()
    }

    /// Vector constructor where collapsible tiles do not need entrophy noise.
    fn new_from_frequency(
        positions: &[GridPosition],
        options_with_weights: BTreeMap<u64, u32>,
    ) -> Vec<GridTile<Self>> {
        let weight_sum: u32 = options_with_weights.values().sum();
        let weight_log_sum = options_with_weights
            .values()
            .map(|v| (*v as f32) * (*v as f32).log2())
            .sum::<f32>();

        positions
            .iter()
            .map(|pos| {
                Self::new_uncollapsed_tile(
                    *pos,
                    options_with_weights.clone(),
                    weight_sum,
                    weight_log_sum,
                    0.0,
                )
            })
            .collect::<Vec<_>>()
    }

    /// Calculate entrophy.
    fn calc_entrophy(&self) -> f32;

    /// Associate function to calculate entrophy.
    fn calc_entrophy_ext(weight_sum: u32, weight_log_sum: f32) -> f32 {
        (weight_sum as f32).log2() - weight_log_sum / (weight_sum as f32)
    }

    fn entrophy_uniform() -> Uniform<f32> {
        Uniform::<f32>::new(0., 0.00001)
    }
}

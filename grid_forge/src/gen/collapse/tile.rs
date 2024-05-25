use std::collections::BTreeMap;

use rand::{
    distributions::{Distribution, Uniform},
    Rng,
};

use crate::{
    map::GridDir,
    tile::{
        identifiable::IdentifiableTileData, GridPosition, GridTile, GridTileRefMut, TileContainer,
        TileData,
    },
};

use super::{error::CollapseErrorKind, rules::AdjacencyRules};
use super::{frequency::FrequencyHints, CollapseError};

/// Tile with options that can be collapsed into one of them.
#[derive(Clone)]
pub struct CollapsibleTileData {
    pub(crate) tile_id: Option<u64>,
    pub(crate) options_with_weights: BTreeMap<u64, u32>,
    weight_sum: u32,
    weight_log_sum: f32,
    entrophy_noise: f32,
}

impl TileData for CollapsibleTileData {}

impl IdentifiableTileData for CollapsibleTileData {
    fn tile_type_id(&self) -> u64 {
        self.tile_id
            .expect("tried to retrieve `tile_id` of uncollapsed tile")
    }
}

impl CollapsibleTileData {
    pub fn new_collapsed(tile_id: u64) -> Self {
        Self {
            tile_id: Some(tile_id),
            options_with_weights: BTreeMap::new(),
            weight_sum: 0,
            weight_log_sum: 0.,
            entrophy_noise: 0.,
        }
    }

    pub fn new_uncollapsed_tile(
        position: GridPosition,
        data: CollapsibleTileData,
    ) -> GridTile<Self> {
        GridTile::new(position, data)
    }
}

impl CollapsibleData for CollapsibleTileData {
    fn have_options(&self) -> bool {
        !self.options_with_weights.is_empty()
    }

    fn remove_option(&mut self, tile_id: u64) {
        if let Some(weight) = self.options_with_weights.remove(&tile_id) {
            self.weight_sum -= weight;
            self.weight_log_sum -= (weight as f32) * (weight as f32).log2()
        }
    }

    fn is_collapsed(&self) -> bool {
        self.tile_id.is_some()
    }

    fn new_collapsed_tile(position: GridPosition, tile_id: u64) -> GridTile<Self> {
        GridTile::new(
            position,
            Self {
                tile_id: Some(tile_id),
                options_with_weights: BTreeMap::new(),
                weight_sum: 0,
                weight_log_sum: 0.,
                entrophy_noise: 0.,
            },
        )
    }

    fn calc_entrophy(&self) -> f32 {
        Self::calc_entrophy_ext(self.weight_sum, self.weight_log_sum) + self.entrophy_noise
    }

    fn new_uncollapsed_tile(
        position: GridPosition,
        options_with_weights: BTreeMap<u64, u32>,
        weight_sum: u32,
        weight_log_sum: f32,
        entrophy_noise: f32,
    ) -> GridTile<Self> {
        GridTile::new(
            position,
            Self {
                tile_id: None,
                options_with_weights,
                weight_sum,
                weight_log_sum,
                entrophy_noise,
            },
        )
    }
}

impl<'a> GridTileRefMut<'a, CollapsibleTileData> {
    pub fn collapse<R: Rng>(&mut self, rng: &mut R) -> Result<bool, CollapseError> {
        if self.inner().is_collapsed() {
            return Ok(false);
        }
        if !self.inner().have_options() {
            return Err(CollapseError::new(
                self.grid_position(),
                CollapseErrorKind::Collapse,
            ));
        }
        let mut current_sum = 0;
        let mut chosen_option = Option::<u64>::None;
        let random = rng.gen_range(0..=self.inner().weight_sum);

        for (option_id, option_weight) in self.inner().options_with_weights.iter() {
            current_sum += option_weight;
            if random <= current_sum {
                chosen_option = Some(*option_id);
                break;
            }
        }

        if let Some(option) = chosen_option {
            self.inner_mut().tile_id = Some(option);
            Ok(true)
        } else {
            unreachable!("should be always possible to collapse!")
        }
    }

    pub fn remove_option(&mut self, tile_id: u64) {
        if let Some(weight) = self.as_mut().options_with_weights.remove(&tile_id) {
            self.as_mut().weight_sum -= weight;
            self.as_mut().weight_log_sum -= (weight as f32) * (weight as f32).log2()
        }
    }

    /// Resolve with regard to adjacency rules if neighbour is collapsed.
    pub(crate) fn resolve_options_neighbour_collapsed<Data>(
        &mut self,
        adjacency: &AdjacencyRules<Data>,
        dir: GridDir,
        neighbour_tile_id: u64,
    ) -> Result<Vec<u64>, GridPosition>
    where
        Data: IdentifiableTileData,
    {
        let mut to_remove = Vec::new();
        for option in self.as_ref().options_with_weights.keys() {
            if !adjacency.is_valid_raw(*option, neighbour_tile_id, dir) {
                to_remove.push(*option);
            }
        }
        for tile_id in to_remove.iter() {
            self.remove_option(*tile_id);
        }
        if !self.as_ref().have_options() {
            return Err(self.grid_position());
        }
        Ok(to_remove)
    }

    /// Resolve with regard to adjacency rules if neighbour is not collapsed.
    pub(crate) fn resolve_options_neighbour_uncollapsed<Data>(
        &mut self,
        adjacency: &AdjacencyRules<Data>,
        dir: GridDir,
        neighbour_options: &[u64],
    ) -> Result<Vec<u64>, GridPosition>
    where
        Data: IdentifiableTileData,
    {
        let mut to_remove = Vec::new();
        for option in self.as_ref().options_with_weights.keys() {
            if neighbour_options
                .iter()
                .all(|neighbour_option| !adjacency.is_valid_raw(*option, *neighbour_option, dir))
            {
                to_remove.push(*option);
            }
        }
        for tile_id in to_remove.iter() {
            self.remove_option(*tile_id);
        }
        if !self.as_ref().have_options() {
            return Err(self.grid_position());
        }
        Ok(to_remove)
    }
}

pub struct CollapsedTileData {
    tile_type_id: u64,
}

impl TileData for CollapsedTileData {}

impl IdentifiableTileData for CollapsedTileData {
    fn tile_type_id(&self) -> u64 {
        self.tile_type_id
    }
}

pub trait CollapsibleData: IdentifiableTileData {
    fn have_options(&self) -> bool;

    fn remove_option(&mut self, tile_id: u64);

    fn is_collapsed(&self) -> bool;

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
        let rng_range = Uniform::<f32>::new(0., 0.00001);
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
}

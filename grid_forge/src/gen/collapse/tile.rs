use std::collections::BTreeMap;

use rand::{
    distributions::{Distribution, Uniform},
    Rng,
};

use crate::{
    gen::adjacency::AdjacencyRules,
    map::GridDir,
    tile::{identifiable::IdentifiableTile, GridTile2D},
    GridPos2D,
};

use super::{frequency::FrequencyHints, CollapseError};

/// Tile with options that can be collapsed into one of them.
pub struct CollapsibleTile {
    pos: GridPos2D,
    pub(crate) tile_id: Option<u64>,
    pub(crate) options_with_weights: BTreeMap<u64, u32>,
    weight_sum: u32,
    weight_log_sum: f32,
    entrophy_noise: f32,
}

impl GridTile2D for CollapsibleTile {
    fn grid_position(&self) -> GridPos2D {
        self.pos
    }

    fn set_grid_position(&mut self, position: GridPos2D) {
        self.pos = position;
    }
}

impl IdentifiableTile for CollapsibleTile {
    fn get_tile_id(&self) -> u64 {
        if let Some(tile_id) = self.tile_id {
            return tile_id;
        }
        panic!(
            "tried to retrieve `tile_id` of uncollapsed tile at position: {:?}",
            self.pos
        );
    }
}

impl CollapsibleTile {
    pub fn new_collapsed(position: GridPos2D, tile_id: u64) -> Self {
        Self {
            pos: position,
            tile_id: Some(tile_id),
            options_with_weights: BTreeMap::new(),
            weight_sum: 0,
            weight_log_sum: 0.,
            entrophy_noise: 0.
        }
    }

    /// Vector constructor where collapsible tiles do not need entrophy noise
    pub fn new_from_frequency<T>(
        positions: &[GridPos2D],
        frequency: &FrequencyHints<T>,
    ) -> Vec<Self>
    where
        T: IdentifiableTile,
    {
        let options_with_weights = frequency.get_all_weights_cloned();
        let weight_sum: u32 = options_with_weights.values().sum();
        let weight_log_sum = options_with_weights
            .values()
            .map(|v| (*v as f32) * (*v as f32).log2())
            .sum::<f32>();

        positions
            .iter()
            .map(|pos| Self {
                pos: *pos,
                tile_id: None,
                options_with_weights: options_with_weights.clone(),
                weight_sum,
                weight_log_sum,
                entrophy_noise: 0.,
            })
            .collect::<Vec<_>>()
    }

    /// Vector constructor where collapsible tiles need entrophy noise
    pub fn new_from_frequency_with_entrophy<T, R>(
        rng: &mut R,
        positions: &[GridPos2D],
        frequency: &FrequencyHints<T>,
    ) -> Vec<Self>
    where
        T: IdentifiableTile,
        R: Rng,
    {
        let rng_range = Uniform::<f32>::new(0., 0.00001);
        let options_with_weights = frequency.get_all_weights_cloned();
        let weight_sum: u32 = options_with_weights.values().sum();
        let weight_log_sum = options_with_weights
            .values()
            .map(|v| (*v as f32) * (*v as f32).log2())
            .sum::<f32>();

        positions
            .iter()
            .map(|pos| Self {
                pos: *pos,
                tile_id: None,
                options_with_weights: options_with_weights.clone(),
                weight_sum,
                weight_log_sum,
                entrophy_noise: rng_range.sample(rng),
            })
            .collect::<Vec<_>>()
    }

    pub fn is_collapsed(&self) -> bool {
        self.tile_id.is_some()
    }

    pub fn have_options(&self) -> bool {
        !self.options_with_weights.is_empty()
    }

    pub fn calc_entrophy(&self) -> f32 {
        (self.weight_sum as f32).log2() - self.weight_log_sum / (self.weight_sum as f32)
            + self.entrophy_noise
    }

    pub fn calc_entrophy_ext(weight_sum: u32, weight_log_sum: f32) -> f32 {
        (weight_sum as f32).log2() - weight_log_sum / (weight_sum as f32)
    }

    pub fn remove_option(&mut self, tile_id: u64) {
        if let Some(weight) = self.options_with_weights.remove(&tile_id) {
            self.weight_sum -= weight;
            self.weight_log_sum -= (weight as f32) * (weight as f32).log2()
        }
    }

    pub fn collapse<R: Rng>(&mut self, rng: &mut R) -> Result<bool, CollapseError> {
        if self.is_collapsed() {
            return Ok(false);
        }
        if !self.have_options() {
            return Err(CollapseError::new_options_empty(self.pos));
        }
        let mut current_sum = 0;
        let mut chosen_option = Option::<u64>::None;
        let random = rng.gen_range(0..=self.weight_sum);

        for (option_id, option_weight) in self.options_with_weights.iter() {
            current_sum += option_weight;
            if random <= current_sum {
                chosen_option = Some(*option_id);
                break;
            }
        }

        if let Some(option) = chosen_option {
            self.tile_id = Some(option);
            Ok(true)
        } else {
            unreachable!("should be always possible to collapse!")
        }
    }

    // --- ADJACENCY RULE --- //
    /// Resolve with regard to adjacency rules if neighbour is collapsed.
    pub fn resolve_options_neighbour_collapsed<T: IdentifiableTile>(
        &mut self,
        adjacency: &AdjacencyRules<T>,
        dir: GridDir,
        neighbour_tile_id: u64,
    ) -> Result<Vec<u64>, CollapseError> {
        let mut to_remove = Vec::new();
        for option in self.options_with_weights.keys() {
            if !adjacency.is_valid_raw(*option, neighbour_tile_id, dir) {
                to_remove.push(*option);
            }
        }
        for tile_id in to_remove.iter() {
            self.remove_option(*tile_id);
        }
        if !self.have_options() {
            return Err(CollapseError::new_options_empty(self.pos));
        }
        Ok(to_remove)
    }

    /// Resolve with regard to adjacency rules if neighbour is not collapsed.
    pub fn resolve_options_neighbour_uncollapsed<T: IdentifiableTile>(
        &mut self,
        adjacency: &AdjacencyRules<T>,
        dir: GridDir,
        neighbour_options: &[u64],
    ) -> Result<Vec<u64>, CollapseError> {
        let mut to_remove = Vec::new();
        for option in self.options_with_weights.keys() {
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
        if !self.have_options() {
            return Err(CollapseError::new_options_empty(self.pos));
        }
        Ok(to_remove)
    }
}

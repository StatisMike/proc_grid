use std::collections::BTreeMap;

use rand::Rng;

use crate::{
    gen::collapse::{error::CollapseErrorKind, tile::CollapsibleData, CollapseError},
    map::GridDir,
    tile::{
        identifiable::{collection::IdentTileCollection, IdentifiableTileData},
        GridPosition, GridTile, GridTileRefMut, TileContainer, TileData,
    },
};

use super::{frequency::PatternAdjacencyRules, pattern::PatternCollection};

pub struct CollapsiblePatternTileData<const P_X: usize, const P_Y: usize, const P_Z: usize> {
    pub(crate) tile_type_id: Option<u64>,
    pub(crate) pattern_id: Option<u64>,
    pub(crate) options_with_weights: BTreeMap<u64, u32>,
    weight_sum: u32,
    weight_log_sum: f32,
    entrophy_noise: f32,
}

impl<const P_X: usize, const P_Y: usize, const P_Z: usize> TileData
    for CollapsiblePatternTileData<P_X, P_Y, P_Z>
{
}

impl<const P_X: usize, const P_Y: usize, const P_Z: usize>
    CollapsiblePatternTileData<P_X, P_Y, P_Z>
{
    pub fn set_weights(&mut self, options_with_weights: BTreeMap<u64, u32>, entrophy_noise: f32) {
        self.options_with_weights = options_with_weights;
        self.weight_sum = self.options_with_weights.values().sum();
        self.weight_log_sum = self
            .options_with_weights
            .values()
            .map(|w| (*w as f32).log2())
            .sum();
        self.entrophy_noise = entrophy_noise;
    }
}

impl<const P_X: usize, const P_Y: usize, const P_Z: usize>
    GridTileRefMut<'_, CollapsiblePatternTileData<P_X, P_Y, P_Z>>
{
    pub fn collapse<R: Rng>(&mut self, rng: &mut R) -> Result<bool, CollapseError> {
        if self.as_ref().pattern_id.is_some() {
            return Ok(false);
        }
        if !self.as_ref().have_options() {
            return Err(CollapseError::new(
                self.grid_position(),
                CollapseErrorKind::Collapse,
            ));
        }
        let mut current_sum = 0;
        let mut chosen_option = Option::<u64>::None;
        let random = rng.gen_range(0..=self.as_ref().weight_sum);

        for (option_id, option_weight) in self.as_ref().options_with_weights.iter() {
            current_sum += option_weight;
            if random <= current_sum {
                chosen_option = Some(*option_id);
                break;
            }
        }

        if let Some(option) = chosen_option {
            self.as_mut().pattern_id = Some(option);
            Ok(true)
        } else {
            unreachable!("should be always possible to collapse!")
        }
    }

    /// Resolve with regard to adjacency rules if neighbour is collapsed.
    pub(crate) fn resolve_options_neighbour_collapsed<Data>(
        &mut self,
        adjacency: &PatternAdjacencyRules<P_X, P_Y, P_Z, Data>,
        dir: GridDir,
        neighbour_tile_id: u64,
    ) -> Result<Vec<u64>, GridPosition>
    where
        Data: IdentifiableTileData,
    {
        let mut to_remove = Vec::new();
        for option in self.as_ref().options_with_weights.keys() {
            if !adjacency
                .as_ref()
                .check_adjacency(option, &dir, &neighbour_tile_id)
            {
                to_remove.push(*option);
            }
        }
        for pattern_id in to_remove.iter() {
            self.as_mut().remove_option(*pattern_id);
        }
        if !self.as_ref().have_options() {
            return Err(self.grid_position());
        }
        Ok(to_remove)
    }

    /// Resolve with regard to adjacency rules if neighbour is not collapsed.
    pub(crate) fn resolve_options_neighbour_uncollapsed<Data>(
        &mut self,
        adjacency: &PatternAdjacencyRules<P_X, P_Y, P_Z, Data>,
        dir: GridDir,
        neighbour_options: &[u64],
    ) -> Result<Vec<u64>, GridPosition>
    where
        Data: IdentifiableTileData,
    {
        let mut to_remove = Vec::new();
        for option in self.as_ref().options_with_weights.keys() {
            if !adjacency.as_ref().check_adjacency_any(option, &dir, neighbour_options)
            {
                to_remove.push(*option);
            }
        }
        for tile_id in to_remove.iter() {
            self.as_mut().remove_option(*tile_id);
        }
        if !self.as_ref().have_options() {
            return Err(self.grid_position());
        }
        Ok(to_remove)
    }
}

impl<const P_X: usize, const P_Y: usize, const P_Z: usize> IdentifiableTileData
    for CollapsiblePatternTileData<P_X, P_Y, P_Z>
{
    fn tile_type_id(&self) -> u64 {
        if let Some(id) = self.tile_type_id {
            return id;
        }
        panic!("attempted to retrieve `tile_type_id` fron uncollapsed PatternTile");
    }
}

impl<const P_X: usize, const P_Y: usize, const P_Z: usize> CollapsibleData
    for CollapsiblePatternTileData<P_X, P_Y, P_Z>
{
    fn is_collapsed(&self) -> bool {
        self.tile_type_id.is_some()
    }

    fn new_collapsed_tile(
        position: crate::tile::GridPosition,
        tile_id: u64,
    ) -> crate::tile::GridTile<Self> {
        GridTile::new(
            position,
            Self {
                tile_type_id: Some(tile_id),
                pattern_id: None,
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

    fn have_options(&self) -> bool {
        !self.options_with_weights.is_empty()
    }

    fn remove_option(&mut self, tile_id: u64) {
        if let Some(weight) = self.options_with_weights.remove(&tile_id) {
            self.weight_sum -= weight;
            self.weight_log_sum -= (weight as f32) * (weight as f32).log2()
        }
    }

    fn new_uncollapsed_tile(
        position: crate::tile::GridPosition,
        options_with_weights: BTreeMap<u64, u32>,
        weight_sum: u32,
        weight_log_sum: f32,
        entrophy_noise: f32,
    ) -> GridTile<Self> {
        GridTile::new(
            position,
            Self {
                tile_type_id: None,
                pattern_id: None,
                options_with_weights: options_with_weights.clone(),
                weight_sum,
                weight_log_sum,
                entrophy_noise,
            },
        )
    }
}

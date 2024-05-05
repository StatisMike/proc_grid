use std::collections::{BTreeMap, VecDeque};

use crate::gen::adjacency::{AdjacencyAnalyzer, AdjacencyRules};
use crate::gen::frequency::FrequencyRules;
use crate::gen::utils::EntrophyQueue;
use crate::map::{GridDir, GridMap2D, GridSize};
use crate::tile::identifiable::builder::IdentTileBuilder;
use crate::tile::{identifiable::IdentifiableTile, GridTile2D};
use crate::GridPos2D;

use super::builder::WFCTileBuilder;

use rand::distributions::{Distribution, Uniform};
use rand::Rng;

#[derive(Clone, Debug)]
pub struct WFCGenTile {
    pos: GridPos2D,
    tile_id: u64,
    options_with_weights: BTreeMap<u64, u32>,
    weight_sum: u32,
    weight_log_sum: f32,
    collapsed: bool,
    entrophy_noise: f32,
}

impl GridTile2D for WFCGenTile {
    fn grid_position(&self) -> GridPos2D {
        self.pos
    }

    fn set_grid_position(&mut self, position: GridPos2D) {
        self.pos = position;
    }
}

impl WFCGenTile {
    fn calc_entrophy(&self) -> f32 {
        (self.weight_sum as f32).log2() - self.weight_log_sum / (self.weight_sum as f32)
            + self.entrophy_noise
    }

    fn calc_entrophy_ext(weight_sum: u32, weight_log_sum: f32) -> f32 {
        (weight_sum as f32).log2() - weight_log_sum / (weight_sum as f32)
    }

    fn remove_option(&mut self, tile_id: u64) {
        if let Some(weight) = self.options_with_weights.remove(&tile_id) {
            self.weight_sum -= weight;
            self.weight_log_sum -= (weight as f32) * (weight as f32).log2()
        }
    }

    fn collapse<R: Rng>(&mut self, rng: &mut R) -> bool {
        if self.collapsed || self.options_with_weights.is_empty() {
            return false;
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
            self.tile_id = option;
            self.collapsed = true;
            self.options_with_weights.clear();
            // println!("collapsed tile at: {:?} with option: {option}", self.pos);
            true
        } else {
            unreachable!("should be always possible to collapse!")
        }
    }

    fn resolve_options_neighbour_collapsed<T: IdentifiableTile>(
        &mut self,
        adjacency: &AdjacencyRules<T>,
        dir: GridDir,
        neighbour_tile_id: u64,
    ) -> bool {
        let mut to_remove = Vec::new();
        for option in self.options_with_weights.keys() {
            if !adjacency.is_valid_raw(*option, neighbour_tile_id, dir) {
                to_remove.push(*option);
            }
        }
        let changed = !to_remove.is_empty();
        for tile_id in to_remove {
            self.remove_option(tile_id);
        }
        changed
    }

    fn resolve_options_neighbour_uncollapsed<T: IdentifiableTile>(
        &mut self,
        adjacency: &AdjacencyRules<T>,
        dir: GridDir,
        neighbour_options: &[u64],
    ) -> bool {
        let mut to_remove = Vec::new();
        for option in self.options_with_weights.keys() {
            if neighbour_options
                .iter()
                .all(|neighbour_option| !adjacency.is_valid_raw(*option, *neighbour_option, dir))
            {
                to_remove.push(*option);
            }
        }
        let changed = !to_remove.is_empty();
        for tile_id in to_remove {
            self.remove_option(tile_id);
        }
        // println!("{:?}", self.options_with_weights);
        changed
    }
}

pub struct WFCResolver<T>
where
    T: IdentifiableTile,
{
    pub(crate) wfc_grid: GridMap2D<WFCGenTile>,
    adjacency_rules: AdjacencyRules<T>,
    frequency_rules: FrequencyRules<T>,
    entrophy_queue: EntrophyQueue,
    changed: VecDeque<GridPos2D>,
}

impl<T> WFCResolver<T>
where
    T: IdentifiableTile,
{
    pub fn new<A>(size: GridSize, analyzer: &A) -> Self
    where
        A: AdjacencyAnalyzer<T>,
    {
        let adjacency_rules = analyzer.adjacency().clone();
        let frequency_rules = analyzer.frequency().clone();

        Self {
            wfc_grid: GridMap2D::new(size),
            adjacency_rules,
            frequency_rules,
            changed: VecDeque::new(),
            entrophy_queue: EntrophyQueue::default(),
        }
    }

    pub fn populate_map_all<R>(&mut self, rng: &mut R)
    where
        R: Rng,
    {
        let all_positions = self.wfc_grid.size().get_all_possible_positions();
        self.populate_map(rng, &all_positions);
    }

    pub fn populate_map<R>(&mut self, rng: &mut R, positions: &[GridPos2D])
    where
        R: Rng,
    {
        let all_weights = self.frequency_rules.get_all_weights_cloned();
        let weight_sum = all_weights.values().sum::<u32>();
        let weight_log_sum = all_weights
            .values()
            .map(|v| (*v as f32) * (*v as f32).log2())
            .sum::<f32>();

        let entrophy_noise_dist = Uniform::<f32>::new(0., 0.0001);
        let entrophy_wo_noise = WFCGenTile::calc_entrophy_ext(weight_sum, weight_log_sum);

        for position in positions {
            let entrophy_noise = entrophy_noise_dist.sample(rng);
            let tile = WFCGenTile {
                pos: *position,
                tile_id: 0,
                options_with_weights: all_weights.clone(),
                weight_sum,
                weight_log_sum,
                collapsed: false,
                entrophy_noise,
            };
            self.wfc_grid.insert_tile(tile);
            self.entrophy_queue
                .insert(*position, entrophy_wo_noise + entrophy_noise);
        }
    }

    pub fn n_resolved(&self) -> usize {
        self.wfc_grid.iter_tiles().filter(|t| t.collapsed).count()
    }

    pub fn n_with_opts(&self) -> usize {
        self.wfc_grid
            .iter_tiles()
            .filter(|t| !t.collapsed && !t.options_with_weights.is_empty())
            .count()
    }

    pub fn n_all(&self) -> usize {
        (self.wfc_grid.size.x() * self.wfc_grid.size.y()) as usize
    }

    fn process_collapse<R: Rng>(&mut self, rng: &mut R) -> bool {
        if let Some(pos) = self.entrophy_queue.pop_next() {
            if let Some(tile) = self.wfc_grid.get_mut_tile_at_position(&pos) {
                if tile.collapse(rng) && !self.changed.contains(&pos) {
                    self.changed.push_back(pos);
                }
            }
            true
        } else {
            false
        }
    }

    pub fn process<R: Rng>(&mut self, rng: &mut R) -> bool {
        let can_continue = self.process_collapse(rng);
        while let Some(changed_pos) = self.changed.pop_front() {
            self.propagate(changed_pos);
        }

        can_continue
    }

    fn propagate(&mut self, pos: GridPos2D) {
        if let Some(tile) = self.wfc_grid.get_tile_at_position(&pos) {
            if tile.collapsed {
                let tile_id = tile.tile_id;
                for direction in GridDir::ALL {
                    if let Some(neighbour) = self.wfc_grid.get_mut_neighbour_at(&pos, direction) {
                        if neighbour.resolve_options_neighbour_collapsed(
                            &self.adjacency_rules,
                            direction.opposite(),
                            tile_id,
                        ) {
                            self.entrophy_queue
                                .insert(neighbour.pos, neighbour.calc_entrophy());
                            if !self.changed.contains(&neighbour.pos) {
                                self.changed.push_back(neighbour.pos);
                            }
                        }
                    }
                }
            } else {
                let tile_options = tile
                    .options_with_weights
                    .keys()
                    .copied()
                    .collect::<Vec<_>>();
                for direction in GridDir::ALL {
                    if let Some(neighbour) = self.wfc_grid.get_mut_neighbour_at(&pos, direction) {
                        if neighbour.pos == (8, 13) {
                            println!("Got it!");
                        }
                        if neighbour.resolve_options_neighbour_uncollapsed(
                            &self.adjacency_rules,
                            direction.opposite(),
                            &tile_options,
                        ) {
                            self.entrophy_queue
                                .insert(neighbour.pos, neighbour.calc_entrophy());
                            if !self.changed.contains(&neighbour.pos) {
                                self.changed.push_back(neighbour.pos);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn build_grid<B>(&self, builder: &B) -> GridMap2D<T>
    where
        B: IdentTileBuilder<T>,
    {
        let size = self.wfc_grid.size();

        let mut map = GridMap2D::new(*size);

        for wfc_tile in self.wfc_grid.iter_tiles() {
            if !wfc_tile.collapsed {
                continue;
            }
            map.insert_tile(builder.create_identifiable_tile(wfc_tile.pos, wfc_tile.tile_id));
        }

        map
    }
}

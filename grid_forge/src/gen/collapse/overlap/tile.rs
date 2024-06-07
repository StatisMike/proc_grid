use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

use private::Sealed;
use rand::distributions::Distribution;
use rand::Rng;

use crate::gen::collapse::error::CollapsedGridError;
use crate::gen::collapse::option::{PerOptionData, WaysToBeOption};
use crate::gen::collapse::{tile::*, CollapsedGrid, CollapsibleGrid, PropagateItem};
use crate::map::{GridDir, GridMap2D, GridSize};
use crate::tile::identifiable::builders::IdentTileBuilder;
use crate::tile::identifiable::collection::IdentTileCollection;
use crate::tile::identifiable::IdentifiableTileData;
use crate::tile::{GridPosition, GridTile, GridTileRef, TileContainer, TileData};

use super::pattern::OverlappingPattern;
use super::{AdjacencyRules, FrequencyHints, PatternCollection};

/// Tile which can be collapsed into one of mutliple [`OverlappingPattern`].
#[derive(Clone, Debug)]
pub struct CollapsiblePattern<P: OverlappingPattern> {
    collapsed_pattern: Option<usize>,
    num_possible_patterns: usize,
    ways_to_be_pattern: WaysToBeOption,
    weight_sum: u32,
    weight_log_sum: f32,
    entrophy_noise: f32,
    pattern_type: PhantomData<P>,
}

impl<P: OverlappingPattern> TileData for CollapsiblePattern<P> {}

impl<P: OverlappingPattern> private::Sealed for CollapsiblePattern<P> {
    fn new_uncollapsed_tile(
        position: GridPosition,
        num_options: usize,
        ways_to_be_option: WaysToBeOption,
        weight_sum: u32,
        weight_log_sum: f32,
        entrophy_noise: f32,
    ) -> GridTile<Self>
    where
        Self: crate::tile::TileData,
    {
        GridTile::new(
            position,
            CollapsiblePattern {
                collapsed_pattern: None,
                num_possible_patterns: num_options,
                ways_to_be_pattern: ways_to_be_option,
                weight_sum,
                weight_log_sum,
                entrophy_noise,
                pattern_type: PhantomData,
            },
        )
    }

    fn ways_to_be_option(&self) -> &WaysToBeOption {
        &self.ways_to_be_pattern
    }

    fn mut_ways_to_be_option(&mut self) -> &mut WaysToBeOption {
        &mut self.ways_to_be_pattern
    }

    fn remove_option(&mut self, weights: (u32, f32)) {
        self.num_possible_patterns -= 1;
        self.weight_sum -= weights.0;
        self.weight_log_sum -= weights.1;
    }

    fn collapse<R: Rng>(
        &mut self,
        rng: &mut R,
        options_data: &crate::gen::collapse::option::PerOptionData,
    ) -> Option<Vec<usize>> {
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
        self.collapsed_pattern = chosen;
        self.num_possible_patterns = 0;
        self.weight_sum = 0;
        self.weight_log_sum = 0.;
        Some(out)
    }
}

impl<P: OverlappingPattern> CollapsibleTileData for CollapsiblePattern<P> {
    fn num_compatible_options(&self) -> usize {
        self.num_possible_patterns
    }

    fn collapse_idx(&self) -> Option<usize> {
        self.collapsed_pattern
    }

    fn new_collapsed_data(option_idx: usize) -> Self {
        Self {
            collapsed_pattern: Some(option_idx),
            num_possible_patterns: 0,
            ways_to_be_pattern: WaysToBeOption::default(),
            weight_sum: 0,
            weight_log_sum: 0.,
            entrophy_noise: 0.,
            pattern_type: PhantomData,
        }
    }

    fn calc_entrophy(&self) -> f32 {
        Self::calc_entrophy_ext(self.weight_sum, self.weight_log_sum) + self.entrophy_noise
    }
}

pub struct CollapsiblePatternGrid<P: OverlappingPattern, Tile: IdentifiableTileData> {
    pub(crate) pattern_grid: GridMap2D<CollapsiblePattern<P>>,
    pub(crate) patterns: PatternCollection<P>,
    pub(crate) option_data: PerOptionData,
    types: PhantomData<(P, Tile)>,
}

impl<P: OverlappingPattern, Tile: IdentifiableTileData> Clone for CollapsiblePatternGrid<P, Tile> {
    fn clone(&self) -> Self {
        Self { pattern_grid: self.pattern_grid.clone(), patterns: self.patterns.clone(), option_data: self.option_data.clone(), types: self.types.clone() }
    }
}

impl<P, Tile> CollapsiblePatternGrid<P, Tile>
where
    Tile: IdentifiableTileData,
    P: OverlappingPattern,
{
    pub fn new_empty(
        size: GridSize,
        patterns: PatternCollection<P>,
        frequencies: &FrequencyHints<P, Tile>,
        adjacencies: &AdjacencyRules<P, Tile>,
    ) -> Result<Self, CollapsedGridError> {
        let mut option_data = PerOptionData::default();
        option_data.populate(&frequencies.get_all_weights_cloned(), adjacencies.inner());

        println!("{}", option_data.get_tile_type_id(&222).unwrap());

        let pattern_ids: HashSet<_> = HashSet::from_iter(patterns.inner().values().map(|p| p.pattern_id()));
        let option_ids: HashSet<_> = HashSet::from_iter(option_data.inner().keys().copied());
        let mut missing_ids = pattern_ids
            .symmetric_difference(&option_ids)
            .copied()
            .collect::<Vec<_>>();
        missing_ids.sort();

        if !missing_ids.is_empty() {
            return Err(CollapsedGridError::new_missing(missing_ids));
        }

        Ok(Self {
            pattern_grid: GridMap2D::new(size),
            patterns,
            option_data,
            types: PhantomData,
        })
    }

    pub fn new_from_collapsed<R: Rng>(
        rng: &mut R,
        collapsed: &CollapsedGrid,
        patterns: PatternCollection<P>,
        frequencies: &FrequencyHints<P, Tile>,
        adjacencies: &AdjacencyRules<P, Tile>,
    ) -> Result<Self, CollapsedGridError> {
        let mut option_data = PerOptionData::default();
        option_data.populate(&frequencies.get_all_weights_cloned(), adjacencies.inner());

        let pattern_ids: HashSet<_> = HashSet::from_iter(patterns.iter_tile_types().copied());
        let option_ids: HashSet<_> = HashSet::from_iter(option_data.inner().keys().copied());
        let mut missing_ids = pattern_ids
            .symmetric_difference(&option_ids)
            .copied()
            .collect::<Vec<_>>();
        missing_ids.sort();
        if !missing_ids.is_empty() {
            return Err(CollapsedGridError::new_missing(missing_ids));
        }

        let mut grid = GridMap2D::new(*collapsed.as_ref().size());

        for tile in
            Self::collapsed_into_collapsible_pattern(rng, collapsed, &patterns, &option_data)?
        {
            grid.insert_tile(tile);
        }

        Ok(Self {
            pattern_grid: grid,
            patterns,
            option_data,
            types: PhantomData,
        })
    }

    fn collapsed_into_collapsible_pattern<R: Rng>(
        rng: &mut R,
        collapsed: &CollapsedGrid,
        patterns: &PatternCollection<P>,
        options: &PerOptionData,
    ) -> Result<Vec<GridTile<CollapsiblePattern<P>>>, CollapsedGridError> {
        let entrophy_uniform = CollapsiblePattern::<P>::entrophy_uniform();
        let ways = options.get_ways_to_become_option();
        let mut out = Vec::new();

        for position in collapsed.as_ref().get_all_positions() {
            let mut possible_patterns = Vec::new();
            let tile_type_id = collapsed
                .as_ref()
                .get_tile_at_position(&position)
                .unwrap()
                .as_ref()
                .tile_type_id();
            'pat_loop: for pattern in patterns.get_patterns_for_tile(tile_type_id) {
                for pos_to_check in P::secondary_tile_positions(&position) {
                    if let Some(tile_type_id) = collapsed
                        .as_ref()
                        .get_tile_at_position(&pos_to_check)
                        .map(|t| t.as_ref().tile_type_id())
                    {
                        if tile_type_id != pattern.get_id_for_pos(&position, &pos_to_check) {
                            continue 'pat_loop;
                        }
                    }
                }

                possible_patterns.push(
                    *options
                        .get_tile_data(&pattern.pattern_id())
                        .expect("cannot get pattern idx"),
                );
            }
            if possible_patterns.is_empty() {
                return Err(CollapsedGridError::new_missing(vec![tile_type_id]));
            }
            let mut current_ways = ways.clone();
            current_ways.purge_others(&possible_patterns);

            let num_options = possible_patterns.len();

            let mut weights = (0u32, 0f32);
            for pattern in possible_patterns {
                let (w, wl) = options.get_weights(pattern);
                weights.0 += w;
                weights.1 += wl;
            }

            out.push(CollapsiblePattern::new_uncollapsed_tile(
                position,
                num_options,
                current_ways,
                weights.0,
                weights.1,
                entrophy_uniform.sample(rng),
            ))
        }

        Ok(out)
    }

    fn retrieve_tile_type_id(&self, tile: &GridTileRef<CollapsiblePattern<P>>) -> Option<u64> {
        match tile.as_ref().collapse_idx() {
            Some(pattern_idx) => {
                let pattern_id = self.option_data.get_tile_type_id(&pattern_idx).unwrap();

                let pattern = self.patterns
                .get_tile_data(&pattern_id)
                .unwrap();
                Some(
                    pattern.tile_type_id(),
                )
            }
            None => None,
        }
    }
}

impl<P: OverlappingPattern, Tile: IdentifiableTileData> CollapsibleGrid<Tile, CollapsiblePattern<P>>
    for CollapsiblePatternGrid<P, Tile>
{
    fn retrieve_collapsed(&self) -> CollapsedGrid {
        let mut out = CollapsedGrid::new(*self.pattern_grid.size());

        for tile in self.pattern_grid.iter_tiles() {
            let Some(tile_type_id) = self.retrieve_tile_type_id(&tile) else {
                continue;
            };

            out.insert_data(&tile.grid_position(), CollapsedTileData::new(tile_type_id));
        }

        out
    }

    fn retrieve_ident<OT: IdentifiableTileData, B: IdentTileBuilder<OT>>(
        &self,
        builder: &B,
    ) -> Result<GridMap2D<OT>, CollapsedGridError> {
        let mut out = GridMap2D::new(*self.pattern_grid.size());

        if let Err(missing) =
            builder.check_missing_ids(&self.patterns.iter_tile_types().copied().collect::<Vec<_>>())
        {
            return Err(CollapsedGridError::new_missing(
                missing.get_missing_tile_type_ids().to_vec(),
            ));
        }

        for tile in self.pattern_grid.iter_tiles() {
            let Some(tile_type_id) = self.retrieve_tile_type_id(&tile) else {
                continue;
            };

            out.insert_tile(builder.build_tile_unchecked(tile.grid_position(), tile_type_id));
        }

        Ok(out)
    }
}

impl<P: OverlappingPattern, Tile: IdentifiableTileData>
    crate::gen::collapse::grid::private::Sealed<CollapsiblePattern<P>>
    for CollapsiblePatternGrid<P, Tile>
{
    fn _grid(&self) -> &GridMap2D<CollapsiblePattern<P>> {
        &self.pattern_grid
    }

    fn _grid_mut(&mut self) -> &mut GridMap2D<CollapsiblePattern<P>> {
        &mut self.pattern_grid
    }

    fn _option_data(&self) -> &PerOptionData {
        &self.option_data
    }

    fn _get_initial_propagate_items(
        &self,
        _to_collapse: &[GridPosition],
    ) -> Vec<crate::gen::collapse::PropagateItem> {
        let mut out = Vec::new();
        let max_options = self.option_data.num_options();

        for pos in self.pattern_grid.get_all_positions() {
            let tile = self.pattern_grid.get_tile_at_position(&pos).unwrap();
            if tile.as_ref().num_compatible_options() == max_options {
                continue;
            }
            let possible =
                HashSet::<_>::from_iter(tile.as_ref().ways_to_be_option().iter_possible());
            for to_remove in (0..max_options).filter(|opt_idx| !possible.contains(opt_idx)) {
                out.push(PropagateItem::new(pos, to_remove));
            }
        }

        out
    }
}

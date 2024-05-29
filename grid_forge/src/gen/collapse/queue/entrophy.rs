use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, HashMap},
};

use rand::Rng;

use super::{CollapseQueue, ResolverSelector};
use crate::gen::collapse::tile::CollapsibleData;
use crate::map::GridMap2D;
use crate::tile::{GridPosition, GridTile, TileContainer};

#[derive(Clone, Copy)]
pub(crate) struct EntrophyItem {
    pos: GridPosition,
    entrophy: f32,
}

impl EntrophyItem {
    pub fn new(pos: GridPosition, entrophy: f32) -> Self {
        Self { pos, entrophy }
    }
}

impl Eq for EntrophyItem {}

impl PartialEq for EntrophyItem {
    fn eq(&self, other: &Self) -> bool {
        self.entrophy == other.entrophy
    }
}

impl PartialOrd for EntrophyItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EntrophyItem {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.entrophy.partial_cmp(&other.entrophy) {
            Some(Ordering::Equal) | None => self.pos.cmp(&other.pos),
            Some(order) => order,
        }
    }
}

/// Select next position to collapse using smallest entrophy condition.
///
/// Its state will be updated every time after tile entrophy changed by removing some of its options.
pub struct EntrophyQueue {
    by_entrophy: BTreeSet<EntrophyItem>,
    by_pos: HashMap<GridPosition, f32>,
    propagation_range: u32,
}

impl EntrophyQueue {
    pub fn new(propagation_range: u32) -> Self {
        Self {
            propagation_range,
            ..Default::default()
        }
    }
}

impl Default for EntrophyQueue {
    fn default() -> Self {
        Self {
            by_entrophy: BTreeSet::new(),
            by_pos: HashMap::new(),
            propagation_range: 5,
        }
    }
}

impl CollapseQueue for EntrophyQueue {
    fn get_next_position(&mut self) -> Option<GridPosition> {
        if let Some(item) = self.by_entrophy.pop_first() {
            self.by_pos.remove(&item.pos);
            return Some(item.pos);
        }
        None
    }

    fn update_queue<Tile, Data>(&mut self, tile: &Tile)
    where
        Tile: TileContainer + AsRef<Data>,
        Data: CollapsibleData,
    {
        let item = EntrophyItem::new(tile.grid_position(), tile.as_ref().calc_entrophy());
        if let Some(existing_entrophy) = self.by_pos.remove(&item.pos) {
            self.by_entrophy
                .remove(&EntrophyItem::new(item.pos, existing_entrophy));
        }
        self.by_pos.insert(item.pos, item.entrophy);
        self.by_entrophy.insert(item);
    }

    fn len(&self) -> usize {
        self.by_entrophy.len()
    }

    fn is_empty(&self) -> bool {
        self.by_entrophy.is_empty()
    }

    fn initialize_queue<T: CollapsibleData>(&mut self, tiles: &[GridTile<T>]) {
        for element in tiles {
            self.update_queue(element)
        }
    }
}

impl ResolverSelector for EntrophyQueue {
    fn populate_inner_grid<R: Rng, Data: CollapsibleData>(
        &mut self,
        rng: &mut R,
        grid: &mut GridMap2D<Data>,
        positions: &[GridPosition],
        options_with_weights: BTreeMap<u64, u32>,
    ) {
        let tiles = Data::new_from_frequency_with_entrophy(rng, positions, options_with_weights);

        self.initialize_queue(&tiles);

        for tile in tiles {
            grid.insert_tile(tile);
        }
    }

    fn needs_update_after_options_change(&self) -> bool {
        true
    }

    fn propagating(&self) -> bool {
        true
    }

    fn in_propagaton_range(&self, collapsed: &GridPosition, candidate: &GridPosition) -> bool {
        collapsed.in_range(candidate, self.propagation_range)
    }
}

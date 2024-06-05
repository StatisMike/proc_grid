use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashMap},
};

use rand::Rng;

use super::CollapseQueue;
use crate::gen::collapse::{option::PerOptionData, tile::CollapsibleTileData};
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
#[derive(Default)]
pub struct EntrophyQueue {
    by_entrophy: BTreeSet<EntrophyItem>,
    by_pos: HashMap<GridPosition, f32>,
}

impl EntrophyQueue {
    pub fn new() -> Self {
        Self::default()
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
        Data: CollapsibleTileData,
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

    fn initialize_queue<T: CollapsibleTileData>(&mut self, tiles: &[GridTile<T>]) {
        for element in tiles {
            self.update_queue(element)
        }
    }
}

impl super::private::Sealed for EntrophyQueue {
    fn populate_inner_grid<R: Rng, Data: CollapsibleTileData>(
        &mut self,
        rng: &mut R,
        grid: &mut GridMap2D<Data>,
        positions: &[GridPosition],
        options_data: &PerOptionData,
    ) {
        let tiles = Data::new_from_frequency_with_entrophy(rng, positions, options_data);

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
}

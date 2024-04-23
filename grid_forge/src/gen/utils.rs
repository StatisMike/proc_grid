use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashMap},
};

use crate::GridPos2D;

struct EntrophyItem {
    pos: GridPos2D,
    entrophy: f32,
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
        if let Some(order) = self.entrophy.partial_cmp(&other.entrophy) {
            return order;
        }
        Ordering::Less
    }
}

pub(crate) struct EntrophyQueue {
    by_entrophy: BTreeSet<EntrophyItem>,
    by_pos: HashMap<GridPos2D, f32>,
}

impl Default for EntrophyQueue {
    fn default() -> Self {
        Self {
            by_entrophy: BTreeSet::new(),
            by_pos: HashMap::new(),
        }
    }
}

impl EntrophyQueue {
    pub fn is_empty(&self) -> bool
    {
        self.by_entrophy.is_empty()
    }
    
    pub fn insert(&mut self, pos: GridPos2D, entrophy: f32) {
        if let Some(existing_entrophy) = self.by_pos.remove(&pos) {
            self.by_entrophy.remove(&EntrophyItem {
                pos,
                entrophy: existing_entrophy,
            });
        }
        self.by_pos.insert(pos, entrophy);
        self.by_entrophy.insert(EntrophyItem { pos, entrophy });
    }

    pub fn pop_next(&mut self) -> Option<GridPos2D> {
        if let Some(item) = self.by_entrophy.pop_first() {
            self.by_pos.remove(&item.pos);
            return Some(item.pos);
        }
        None
    }
}

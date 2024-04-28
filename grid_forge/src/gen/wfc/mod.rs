use std::hash::{DefaultHasher, Hash, Hasher};

use crate::tile::GridTile2D;

pub mod analyzer;
pub mod builder;
pub mod resolver;

#[cfg(feature = "vis")]
pub mod vis;

pub trait WFCTile
where
    Self: GridTile2D,
{
    fn wfc_id(&self) -> u64;
}

impl<T> WFCTile for T
where
    T: GridTile2D + Hash + Clone,
{
    fn wfc_id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        let mut cloned = self.clone();
        cloned.set_grid_position((0, 0));
        cloned.hash(&mut hasher);
        hasher.finish()
    }
}

pub mod error;
pub mod map;
pub mod tile;

#[cfg(feature = "gen")]
pub mod gen;

pub type GridPos2D = (u32, u32);

pub fn add_grid_positions(g1: GridPos2D, g2: GridPos2D) -> GridPos2D {
    (g1.0 + g2.0, g1.1 + g2.1)
}

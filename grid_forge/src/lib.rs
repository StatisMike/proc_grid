pub mod error;
pub mod map;
pub mod tile;
pub(crate) mod utils;

#[allow(clippy::non_minimal_cfg)]
#[cfg(any(feature = "godot"))]
pub(crate) mod ext;

#[cfg(feature = "godot")]
pub mod godot {
    use crate::ext;
    pub use ext::godot;
}

#[cfg(feature = "vis")]
pub mod vis;

#[cfg(feature = "gen")]
pub mod gen;

pub type GridPos2D = (u32, u32);

// pub struct GridPos2D(u32, u32);

pub fn add_grid_positions(g1: GridPos2D, g2: GridPos2D) -> GridPos2D {
    (g1.0 + g2.0, g1.1 + g2.1)
}

pub fn gen_grid_positions_square(upper_left: GridPos2D, lower_right: GridPos2D) -> Vec<GridPos2D> {
    let mut out = Vec::new();

    for y in upper_left.1..=lower_right.1 {
        for x in upper_left.0..=lower_right.0 {
            out.push((x, y));
        }
    }

    out
}

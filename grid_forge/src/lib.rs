pub mod error;
pub mod map;
pub mod tile;

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

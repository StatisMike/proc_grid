//! Overlapping procedural generation algorithm.
//!
//! Implementation is analogous to the one named `Overlapping Wave-Function Collapse` in external sources.

mod analyzer;
mod pattern;
mod resolver;
mod tile;

pub use {analyzer::*, pattern::*, resolver::*, tile::*};

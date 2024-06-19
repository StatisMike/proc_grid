//! # Singular procedural generation
//! 
//! Collapsible generation algorithm working with rulesets that are based on tile type adjacencies. Consistently named within other
//! sources as *single-tiled* algorithms.
//! 
//! As it is based on tile type adjacencies, it is more viable to describe the rules manually than pattern-based 
//! [`overlap`](crate::gen::collapse::overlap).
//! 
//! ## Struct types
//! 
//! In general the types are described in the documentation for [`collapse`](crate::gen::collapse) module.
//! 
//! - [`AdjacencyRules`] and [`FrequencyHints`] are self-descriptive. The latter are not produced by the *analyzer*, but the method
//! for their derivation from the sample gridmap is exposed..
//! - [`Analyzer`] is a trait implemented by two distincts analyzers. The [`IdentityAnalyzer`] in general produced more restrictive rules,
//! as it search for exact neigbours on the sample gridmap. The [`BorderAnalyzer`] is more liberal, as it takes an extra step and derives
//! more rules based on the distinct tile borders, making additional options available if they *could be* placed on the sample gridmap
//! next to each other.
//! - [`CollapsibleTileGrid`] is the collection of [`CollapsibleTile`].
//! - [`Resolver`] is the main executor of the algorithm.

mod analyzer;
mod resolver;
mod tile;

pub use {analyzer::*, resolver::*, tile::*};

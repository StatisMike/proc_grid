//! # Overlapping collapsible generation
//!
//! Collapsible generation algorithm working with rulesets that are based on patterns of tile type patterns. Often named
//! within other sources as *overlapping Wave-Function Collapse*.
//!
//! As it is based on multi-tile patterns, it is useful for generation where tiles itself don't have a clear adjacencies
//! relation, but instead are grouped in meaningful patterns.
//!
//! ## Struct types
//!
//! In general the types are described in the documentation for [`collapse`](crate::gen::collapse) module.
//!
//! - [`AdjacencyRules`] and [`FrequencyHints`] are self-descriptive, and both can be created by the analyzer.
//! - [`Analyzer`] can generate [`AdjacencyRules`], [`FrequencyHints`] and [`PatternCollection`].
//! - [`PatternCollection`] is a collection of [`OverlappingPattern`]s gathered from sample maps. It is an additional
//! element over [`singular`](crate::gen::collapse::singular) workflow needed for translation between collapsed patterns
//! and underlying individual `tile_type_id`.
//! - [`CollapsiblePatternGrid`] is the collection of [`CollapsiblePattern`], and is used by [`Resolver`] to generate
//! new map.

mod analyzer;
mod pattern;
mod resolver;
mod tile;

pub use {analyzer::*, pattern::*, resolver::*, tile::*};

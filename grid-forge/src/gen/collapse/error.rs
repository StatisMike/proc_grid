use std::{error::Error, fmt::Display};

use crate::{map::GridSize, tile::GridPosition};

/// Error occuring during collapse process.
///
/// This error is returned by *resolvers*, when the collapse process encounters a contradiction, seen as an uncollapsed tile
/// with no possible options left. It can occur because of too restrictive ruleset, unsound *queue* being chosen, or just
/// randomly.
///
/// There are some methods available to get more information about the type and possible fallback solutions:
/// - [`CollapseError::failed_pos()`] returns [`GridPosition`] of tile which caused the error, while [`CollapseError::failed_iter()`]
/// returns the count of successful collapse iterations before the error occured. If the same position fails consistently
/// on multiple retries or failure occurs just at the beginning of the process, most likely the rulesets are
/// too restrictive. In this case you can try to increase the number of analyzed samples, try to modify used *adjacency rules* or
/// tweak the *frequency hints*.
/// - [`CollapseError::is_probabilistic()`] returns `true` if the error can be solved by retrying the operation. If the error
/// occurs at the sole beginning of the process, before first successful collapse it is deemed not probabilistic and is most likely caused
/// by placing some incompatible pre-collapsed tiles in *collapsible grid* provided to the *resolver*.
#[derive(Debug)]
pub struct CollapseError {
    pos: GridPosition,
    kind: CollapseErrorKind,
    iter: u32,
}

impl CollapseError {
    pub(crate) fn new(pos: GridPosition, kind: CollapseErrorKind, iter: u32) -> Self {
        Self { pos, kind, iter }
    }

    #[inline(always)]
    pub(crate) fn from_result<T>(
        result: Result<T, GridPosition>,
        kind: CollapseErrorKind,
        iter: u32,
    ) -> Result<T, Self> {
        match result {
            Ok(val) => Ok(val),
            Err(pos) => Err(CollapseError::new(pos, kind, iter)),
        }
    }

    /// Returns [`GridPosition`] of tile which caused the error.
    pub fn failed_pos(&self) -> GridPosition {
        self.pos
    }

    /// Returns `true` if the error can be solved by retrying the operation.
    pub fn is_probabilistic(&self) -> bool {
        !matches!(self.kind, CollapseErrorKind::Init)
    }

    /// Returns iteration number when the error occured.
    pub fn failed_iter(&self) -> u32 {
        self.iter
    }
}

impl Display for CollapseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            CollapseErrorKind::Collapse => write!(
                f,
                "tile at position: {:?} have no options left while collapsing on iteration {}!",
                self.pos, self.iter
            ),
            CollapseErrorKind::Init => write!(
                f,
                "tile at position: {:?} have no options left during initial option removal!",
                self.pos,
            ),
            CollapseErrorKind::Propagation => write!(
                f,
                "tile at position: {:?} have no options left during propagation on iteration {}!",
                self.pos, self.iter
            ),
        }
    }
}

impl Error for CollapseError {}

#[derive(Debug)]
pub(crate) enum CollapseErrorKind {
    Collapse,
    Init,
    Propagation,
}

/// Error occuring during the operations on *collapsible grids*.
///
/// Indicates the inconsistency between provided [`CollapsedGrid`](crate::gen::collapse::CollapsedGrid) and target grid,
/// either because of the `tile_type_ids` being not consistent beetween collapsed tiles and provided rulesets, or
/// because the grid size incompatibility.
#[derive(Debug)]
pub struct CollapsibleGridError {
    missing_type_ids: Option<Vec<u64>>,
    sizes: Option<(GridSize, GridSize)>,
    position: Option<GridPosition>,
}

impl CollapsibleGridError {
    pub(crate) fn new_missing(missing_type_ids: Vec<u64>) -> Self {
        Self {
            missing_type_ids: Some(missing_type_ids),
            sizes: None,
            position: None,
        }
    }
    pub(crate) fn new_wrong_size(source_size: GridSize, target_size: GridSize) -> Self {
        Self {
            missing_type_ids: None,
            sizes: Some((source_size, target_size)),
            position: None,
        }
    }
    pub(crate) fn new_collapse(position: GridPosition) -> Self {
        Self {
            missing_type_ids: None,
            sizes: None,
            position: Some(position),
        }
    }

    /// If error originates from missing types during transforming [`GridMap2D`](crate::map::GridMap2D) of
    /// [`CollapsedTileData`](crate::gen::collapse::tile::CollapsedTileData) into [`CollapsibleGrid`](crate::gen::collapse::grid::CollapsibleGrid),
    /// it will contain vector of `tile_type_ids` which were missing.
    pub fn missing_type_ids(&self) -> &Option<Vec<u64>> {
        &self.missing_type_ids
    }

    /// If error originates from incompatible [`GridSize`] of source [`GridMap2D`](crate::map::GridMap2D) and target
    /// [`CollapsibleGrid`](crate::gen::collapse::grid::CollapsibleGrid), it will contain tuple of (`source_size`, `target_size`).
    pub fn sizes(&self) -> &Option<(GridSize, GridSize)> {
        &self.sizes
    }

    /// If error originates from incompatible prepopulated [`CollapsedTileData`](crate::gen::collapse::CollapsedTileData) during their transformation
    /// into [`CollapsiblePatternGrid`](crate::gen::collapse::overlap::CollapsiblePatternGrid), it will contain the position of problematic tile.
    pub fn position(&self) -> &Option<GridPosition> {
        &self.position
    }
}

impl Display for CollapsibleGridError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.missing_type_ids, &self.sizes, &self.position) {
            (Some(missing), None, None) => write!(f, "there are {} `tile_type_ids` missing from underlying CollapsibleGrid data. Make sure that the `CollapsibleGrid` have been provided correct rulesets", missing.len()),
            (None, Some((source, target)), None) => write!(f, "size of source `GridMap`: {source:?} is greater than target `CollapsibleGrid`: {target:?}"),
            (None, None, Some(position)) => write!(f, "tile at position: {position:?} cannot get any compatible patterns"),
            _ => unreachable!("either created by `Self::new_missing()` or `Self::new_wrong_size()`"),
        }
    }
}

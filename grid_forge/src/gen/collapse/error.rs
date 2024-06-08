use std::{error::Error, fmt::Display};

use crate::{map::GridSize, tile::GridPosition};

#[derive(Debug)]
pub struct CollapseError {
    pos: GridPosition,
    kind: CollapseErrorKind,
}

impl CollapseError {
    pub(crate) fn new(pos: GridPosition, kind: CollapseErrorKind) -> Self {
        Self { pos, kind }
    }

    pub(crate) fn from_result<T>(
        result: Result<T, GridPosition>,
        kind: CollapseErrorKind,
    ) -> Result<T, Self> {
        match result {
            Ok(val) => Ok(val),
            Err(pos) => Err(CollapseError::new(pos, kind)),
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
}

impl Display for CollapseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            CollapseErrorKind::Collapse => write!(
                f,
                "tile at position: {:?} have no options left while collapsing!",
                self.pos
            ),
            CollapseErrorKind::NeighbourUpdate => write!(
                f,
                "tile at position: {:?} have no options left after collapsed neighbours update!",
                self.pos
            ),
            CollapseErrorKind::Init => write!(
                f,
                "tile at position: {:?} have no options left during initial option removal!",
                self.pos
            ),
            CollapseErrorKind::Propagation => write!(
                f,
                "tile at position: {:?} have no options left during propagation!",
                self.pos
            ),
        }
    }
}

impl Error for CollapseError {}

#[derive(Debug)]
pub(crate) enum CollapseErrorKind {
    Collapse,
    NeighbourUpdate,
    Init,
    Propagation,
}

#[derive(Debug)]
pub struct CollapsedGridError {
    missing_type_ids: Option<Vec<u64>>,
    sizes: Option<(GridSize, GridSize)>,
    position: Option<GridPosition>
}

impl CollapsedGridError {
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
            position: Some(position)
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

impl Display for CollapsedGridError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.missing_type_ids, &self.sizes, &self.position) {
            (Some(missing), None, None) => write!(f, "there are {} `tile_type_ids` missing from underlying CollapsibleGrid data. Make sure that the `CollapsibleGrid` have been provided correct rulesets", missing.len()),
            (None, Some((source, target)), None) => write!(f, "size of source `GridMap`: {source:?} is greater than target `CollapsibleGrid`: {target:?}"),
            (None, None, Some(position)) => write!(f, "tile at position: {position:?} cannot get any compatible patterns"),
            _ => unreachable!("either created by `Self::new_missing()` or `Self::new_wrong_size()`"),
        }
    }
}

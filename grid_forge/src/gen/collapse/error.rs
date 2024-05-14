use std::fmt::Display;

use crate::GridPosition;

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

    pub fn failed_pos(&self) -> GridPosition {
        self.pos
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

#[derive(Debug)]
pub(crate) enum CollapseErrorKind {
    Collapse,
    NeighbourUpdate,
    Init,
    Propagation,
}

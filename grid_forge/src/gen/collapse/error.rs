use std::fmt::Display;

use crate::GridPos2D;

#[derive(Debug)]
pub struct CollapseError {
    pos: GridPos2D,
    kind: CollapseErrorKind,
}

impl CollapseError {
    pub(crate) fn new(pos: GridPos2D, kind: CollapseErrorKind) -> Self {
        Self { pos, kind }
    }

    pub(crate) fn from_result<T>(
        result: Result<T, GridPos2D>,
        kind: CollapseErrorKind,
    ) -> Result<T, Self> {
        match result {
            Ok(val) => Ok(val),
            Err(pos) => Err(CollapseError::new(pos, kind)),
        }
    }

    pub fn failed_pos(&self) -> GridPos2D {
        self.pos
    }
}

impl Display for CollapseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            CollapseErrorKind::OnCollapse => write!(
                f,
                "tile at position: {:?} have no options left while collapsing!",
                self.pos
            ),
            CollapseErrorKind::OnNeighbourUpdate => write!(
                f,
                "tile at position: {:?} have no options left after collapsed neighbours update!",
                self.pos
            ),
            CollapseErrorKind::OnInit => write!(
                f,
                "tile at position: {:?} have no options left during initial option removal!",
                self.pos
            ),
            CollapseErrorKind::OnPropagation => write!(
                f,
                "tile at position: {:?} have no options left during propagation!",
                self.pos
            ),
        }
    }
}

#[derive(Debug)]
pub(crate) enum CollapseErrorKind {
    OnCollapse,
    OnNeighbourUpdate,
    OnInit,
    OnPropagation,
}

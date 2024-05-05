use std::fmt::Display;

use crate::GridPos2D;

pub mod frequency;
pub mod queue;
pub mod resolver;
pub mod rules;
pub mod tile;

#[derive(Debug)]
pub struct CollapseError {
    kind: CollapseErrorKind,
}

impl CollapseError {
    pub(crate) fn new_options_empty(pos: GridPos2D) -> Self {
        Self {
            kind: CollapseErrorKind::OptionsEmpty(pos),
        }
    }

    pub fn failed_pos(&self) -> GridPos2D {
        match self.kind {
            CollapseErrorKind::OptionsEmpty(pos) => pos,
        }
    }
}

impl Display for CollapseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            CollapseErrorKind::OptionsEmpty(pos) => write!(
                f,
                "cannot collapse: tile at position: {pos:?} have no options left"
            ),
        }
    }
}

#[derive(Debug)]
enum CollapseErrorKind {
    OptionsEmpty(GridPos2D),
}

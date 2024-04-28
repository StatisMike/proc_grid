use std::fmt::Display;

use crate::GridPos2D;

#[derive(Debug, Clone)]
pub struct VisError<const WIDTH: usize, const HEIGHT: usize> {
    kind: VisErrorKind,
}

impl<const WIDTH: usize, const HEIGHT: usize> VisError<WIDTH, HEIGHT> {
    pub(crate) fn new_nonexist(pos: GridPos2D) -> Self {
        Self {
            kind: VisErrorKind::NonExistingTile(pos),
        }
    }

    pub(crate) fn new_tile(x: u32, y: u32) -> Self {
        Self {
            kind: VisErrorKind::WrongSizeTile { x, y },
        }
    }

    pub(crate) fn new_grid_load(x: u32, y: u32) -> Self {
        Self {
            kind: VisErrorKind::WrongSizeGridLoad { x, y },
        }
    }

    pub(crate) fn new_grid_save(expected: (u32, u32), actual: (u32, u32)) -> Self {
        Self {
            kind: VisErrorKind::WrongSizeGridSave { expected, actual },
        }
    }

    pub(crate) fn new_nopix(tile_id: u64) -> Self {
        Self {
            kind: VisErrorKind::NoPixelsForIdent(tile_id),
        }
    }

    pub(crate) fn new_io(read: bool, tile_pos: GridPos2D, pixel_pos: (u32, u32)) -> Self {
        if read {
            Self {
                kind: VisErrorKind::PixelRead {
                    tile_pos,
                    pixel_pos,
                },
            }
        } else {
            Self {
                kind: VisErrorKind::PixelWrite {
                    tile_pos,
                    pixel_pos,
                },
            }
        }
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> Display for VisError<WIDTH, HEIGHT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
          VisErrorKind::NonExistingTile(pos) => {
            write!(f, "tile at position: {pos:?} is not contained within used `VisCollection`. Make sure to register it first manually")
          }
            VisErrorKind::WrongSizeTile { x, y } => {
                write!(f, "expected tile pixel size (x: {WIDTH}; y: {HEIGHT}) is incompatible with actual image size: (x: {x}, y: {y})")
            }
            VisErrorKind::WrongSizeGridLoad { x, y } => {
                write!(f, "expected tile pixel size (x: {WIDTH}; y: {HEIGHT}) is incompatible with GridMap image size: (x: {x}, y: {y})")
            }
            VisErrorKind::NoPixelsForIdent(tile_id) => write!(
              f,
              "cannot draw tile: no pixels for tile of id: {tile_id} is present"
          ),
      VisErrorKind::PixelRead {
          tile_pos,
          pixel_pos,
      } => write!(f, "cannot read tile pixels: image buffer is out of bounds for tile on position: {tile_pos:?}, with pixel: {pixel_pos:?}"),
      VisErrorKind::PixelWrite {
          tile_pos,
          pixel_pos,
      } => write!(f, "cannot draw tile: image buffer is out of bounds for tile on position: {tile_pos:?}, with pixel: {pixel_pos:?}"),
            VisErrorKind::WrongSizeGridSave { expected, actual } => write!(f, "actual image buffer size: {actual:?} differs from expected: {expected:?}"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum VisErrorKind {
    NonExistingTile(GridPos2D),
    NoPixelsForIdent(u64),
    PixelRead {
        tile_pos: GridPos2D,
        pixel_pos: (u32, u32),
    },
    PixelWrite {
        tile_pos: GridPos2D,
        pixel_pos: (u32, u32),
    },
    WrongSizeTile {
        x: u32,
        y: u32,
    },
    WrongSizeGridLoad {
        x: u32,
        y: u32,
    },
    WrongSizeGridSave {
        expected: (u32, u32),
        actual: (u32, u32),
    },
}

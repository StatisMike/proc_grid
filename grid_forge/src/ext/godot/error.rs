use std::error::Error;
use std::fmt::Display;

use crate::tile::identifiable::builders::TileBuilderError;

use super::collection::GodotTileMapTileInfo;

#[derive(Clone, Debug)]
pub struct GodotTileError {
    kind: GodotTileErrorKind,
}

impl GodotTileError {
    pub fn new_no_id_for_info(tile_info: GodotTileMapTileInfo) -> Self {
        Self {
            kind: GodotTileErrorKind::NoTileForInfo(tile_info),
        }
    }

    pub fn new_no_info_for_id(tile_type_id: u64) -> Self {
        Self {
            kind: GodotTileErrorKind::NoInfoForTile(tile_type_id),
        }
    }
}

impl Display for GodotTileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            GodotTileErrorKind::NoTileForInfo(info) => {
                write!(f, "cannot get `tile_type_id` for tile info: {info:?}")
            }
            GodotTileErrorKind::NoInfoForTile(id) => {
                write!(f, "cannot get `GodotTileInfo` for `tile_type_id`: {id}")
            }
            GodotTileErrorKind::Builder(err) => err.fmt(f),
        }
    }
}

impl Error for GodotTileError {}

impl From<TileBuilderError> for GodotTileError {
    fn from(value: TileBuilderError) -> Self {
        Self {
            kind: GodotTileErrorKind::Builder(value),
        }
    }
}

#[derive(Clone, Debug)]
enum GodotTileErrorKind {
    NoTileForInfo(GodotTileMapTileInfo),
    NoInfoForTile(u64),
    Builder(TileBuilderError),
}

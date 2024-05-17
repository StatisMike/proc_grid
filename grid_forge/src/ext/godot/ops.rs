use godot::{builtin::Rect2i, engine::TileMap, obj::Gd};

use crate::{
    map::{GridMap2D, GridSize},
    tile::{
        identifiable::{
            builders::IdentTileBuilder, collection::IdentTileCollection, IdentifiableTile,
            IdentifiableTileData,
        },
        GridPosition,
    },
};

use super::{
    collection::{GodotInfoBuilder, GodotTileMapCollection},
    error::GodotTileError,
};

pub fn load_gridmap_from_tilemap_auto<Data: IdentifiableTileData, B: IdentTileBuilder<Data>>(
    tilemap: &Gd<TileMap>,
    collection: &mut GodotTileMapCollection,
    builder: &B,
) -> Result<GridMap2D<Data>, GodotTileError> {
    let size = size_from_rect(tilemap.get_used_rect());

    let mut grid_map = GridMap2D::new(size);
    let mut sources = GodotInfoBuilder::default();

    for coords in tilemap.get_used_cells(0).iter_shared() {
        let source_id = tilemap.get_cell_source_id(0, coords);

        let info = sources.build_from_tilemap(source_id, tilemap, coords);
        let tile_type_id = info.get_tile_type_id();

        collection.add_tile_data(tile_type_id, info);

        let tile = builder.build_tile(GridPosition::from_godot_v2i(coords), tile_type_id)?;
        grid_map.insert_tile(tile);
    }
    Ok(grid_map)
}

pub fn load_gridmap_from_tilemap_manual<Data: IdentifiableTileData, B: IdentTileBuilder<Data>>(
    tilemap: &Gd<TileMap>,
    collection: &GodotTileMapCollection,
    builder: &B,
) -> Result<GridMap2D<Data>, GodotTileError> {
    let size = size_from_rect(tilemap.get_used_rect());

    let mut grid_map = GridMap2D::new(size);
    let mut sources = GodotInfoBuilder::default();

    for coords in tilemap.get_used_cells(0).iter_shared() {
        let source_id = tilemap.get_cell_source_id(0, coords);

        let info = sources.build_from_tilemap(source_id, tilemap, coords);

        if let Some(tile_type_id) = collection.get_tile_type_id(&info) {
            let tile = builder.build_tile(GridPosition::from_godot_v2i(coords), tile_type_id)?;
            grid_map.insert_tile(tile);
        } else {
            return Err(GodotTileError::new_no_id_for_info(info));
        }
    }

    Ok(grid_map)
}

pub fn write_gridmap_to_tilemap<Data: IdentifiableTileData>(
    gridmap: &GridMap2D<Data>,
    tilemap: &mut Gd<TileMap>,
    collection: &GodotTileMapCollection,
) -> Result<(), GodotTileError> {
    for position in gridmap.get_all_positions() {
        let tile = gridmap
            .get_tile_at_position(&position)
            .expect("cannot get tile!");

        if let Some(godot_info) = collection.get_tile_data(&tile.tile_type_id()) {
            godot_info.insert_to_tilemap(
                tilemap,
                position.get_godot_coords(),
                position.get_godot_layer().unwrap_or_else(|| 0),
            )
        } else {
            return Err(GodotTileError::new_no_info_for_id(tile.tile_type_id()));
        }
    }
    Ok(())
}

fn size_from_rect(rect: Rect2i) -> GridSize {
    GridSize::new_xy(
        (rect.size.x - rect.position.x) as u32,
        (rect.size.y - rect.position.y) as u32,
    )
}

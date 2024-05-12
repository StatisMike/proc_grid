use godot::{
    builtin::{Rect2i, Vector2i},
    engine::TileMap,
    obj::Gd,
};

use crate::{
    map::{GridMap2D, GridSize},
    tile::identifiable::{
        builders::IdentTileBuilder, collection::IdentTileCollection, IdentifiableTile,
    },
};

use super::{
    collection::{GodotInfoBuilder, GodotTileMapCollection},
    error::GodotTileError,
};

pub fn load_gridmap_from_tilemap_auto<T: IdentifiableTile, B: IdentTileBuilder<T>>(
    tilemap: &Gd<TileMap>,
    collection: &mut GodotTileMapCollection,
    builder: &B,
) -> Result<GridMap2D<T>, GodotTileError> {
    let size = size_from_rect(tilemap.get_used_rect());

    let mut grid_map = GridMap2D::new(size);
    let mut sources = GodotInfoBuilder::default();

    for coord in tilemap.get_used_cells(0).iter_shared() {
        let source_id = tilemap.get_cell_source_id(0, coord);

        let info = sources.build_from_tilemap(source_id, tilemap, coord);
        let tile_type_id = info.get_tile_type_id();

        collection.add_tile_data(tile_type_id, info);

        let tile = builder.build_tile((coord.x as u32, coord.y as u32), tile_type_id)?;
        grid_map.insert_tile(tile);
    }
    Ok(grid_map)
}

pub fn load_gridmap_from_tilemap_manual<T: IdentifiableTile, B: IdentTileBuilder<T>>(
    tilemap: &Gd<TileMap>,
    collection: &GodotTileMapCollection,
    builder: &B,
) -> Result<GridMap2D<T>, GodotTileError> {
    let size = size_from_rect(tilemap.get_used_rect());

    let mut grid_map = GridMap2D::new(size);
    let mut sources = GodotInfoBuilder::default();

    for coord in tilemap.get_used_cells(0).iter_shared() {
        let source_id = tilemap.get_cell_source_id(0, coord);

        let info = sources.build_from_tilemap(source_id, tilemap, coord);

        if let Some(tile_type_id) = collection.get_tile_type_id(&info) {
            let tile = builder.build_tile((coord.x as u32, coord.y as u32), tile_type_id)?;
            grid_map.insert_tile(tile);
        } else {
            return Err(GodotTileError::new_no_id_for_info(info));
        }
    }

    Ok(grid_map)
}

pub fn write_gridmap_to_tilemap<T: IdentifiableTile>(
    gridmap: &GridMap2D<T>,
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
                Vector2i::new(position.0 as i32, position.1 as i32),
                0,
            )
        } else {
            return Err(GodotTileError::new_no_info_for_id(tile.tile_type_id()));
        }
    }
    Ok(())
}

fn size_from_rect(rect: Rect2i) -> GridSize {
    GridSize::new(
        (rect.size.x - rect.position.x) as u32,
        (rect.size.y - rect.position.y) as u32,
    )
}

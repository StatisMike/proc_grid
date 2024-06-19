use godot::builtin::Rect2i;
use godot::classes::TileMap;
use godot::obj::Gd;

use crate::map::{GridMap2D, GridSize};
use crate::tile::identifiable::builders::IdentTileBuilder;
use crate::tile::identifiable::collection::IdentTileCollection;
use crate::tile::identifiable::IdentifiableTileData;
use crate::tile::GridPosition;

use super::collection::{GodotInfoBuilder, GodotTileMapCollection};
use super::error::GodotTileError;

/// Loads [`GridMap2D`] from [`TileMap`], automatically loading read tiles into [`GodotTileMapCollection`].
/// 
/// Currently supports only `layer = 0`.
/// 
/// Automatic character of the process means that:
/// - not all tiles from underlying [`TileSet`](godot::classes::TileSet) sources will be loaded into [`GodotTileMapCollection`], only 
/// those which are used in the [`TileMap`].
/// - `tile_type_id` for each tile will be automatically generated.
/// 
/// If more control over the process is needed, [`load_gridmap_from_tilemap_manual`] can be used.
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

/// Loads [`GridMap2D`] from [`TileMap`], using tiles collected in [`GodotTileMapCollection`].
/// 
/// Currently supports only `layer = 0`.
/// 
/// As the process is manual, the `tile_type_id` for each tile will be taken from the collection, not generated
/// automatically. Process can fail if the collection does not contain the required tile.
/// 
/// If such control over the process is not needed, [`load_gridmap_from_tilemap_auto`] can be used.
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

/// Writes [`GridMap2D`] to [`TileMap`], using [`GodotTileMapTileInfo`](crate::godot::GodotTileMapTileInfo) from [`GodotTileMapCollection`].
/// 
/// Currently supports only `layer = 0`.
pub fn write_gridmap_to_tilemap<Data: IdentifiableTileData>(
    gridmap: &GridMap2D<Data>,
    tilemap: &mut Gd<TileMap>,
    collection: &GodotTileMapCollection,
) -> Result<(), GodotTileError> {
    for position in gridmap.get_all_positions() {
        let tile = gridmap
            .get_tile_at_position(&position)
            .expect("cannot get tile!");

        if let Some(godot_info) = collection.get_tile_data(&tile.as_ref().tile_type_id()) {
            godot_info.insert_to_tilemap(
                tilemap,
                position.get_godot_coords(),
                position.get_godot_layer().unwrap_or(0),
            )
        } else {
            return Err(GodotTileError::new_no_info_for_id(
                tile.as_ref().tile_type_id(),
            ));
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

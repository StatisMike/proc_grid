use std::collections::{hash_map::Entry, HashMap};

use godot::{
    builtin::{Rect2i, Vector2i},
    engine::{TileMap, TileSetAtlasSource, TileSetScenesCollectionSource},
    obj::Gd,
};

use crate::{
    map::{GridMap2D, GridSize},
    tile::identifiable::{
        builders::IdentTileBuilder,
        collection::IdentTileCollection,
        IdentifiableTile,
    },
};

use super::{
    collection::{GodotTileCollection, GodotTileInfo},
    error::GodotTileError,
    TileSourceType,
};

pub fn load_gridmap_from_tilemap_auto<T: IdentifiableTile, B: IdentTileBuilder<T>>(
    tilemap: &Gd<TileMap>,
    collection: &mut GodotTileCollection,
    builder: &B,
) -> Result<GridMap2D<T>, GodotTileError> {
    let size = size_from_rect(tilemap.get_used_rect());

    let mut grid_map = GridMap2D::new(size);
    let mut sources = InfoBuilder::default();

    for coord in tilemap.get_used_cells(0).iter_shared() {
        let source_id = tilemap.get_cell_source_id(0, coord);

        let info = sources.build(source_id, tilemap, coord);
        let tile_type_id = info.get_tile_type_id();

        collection.add_tile_data(tile_type_id, info);

        let tile = builder.build_tile((coord.x as u32, coord.y as u32), tile_type_id)?;
        grid_map.insert_tile(tile);
    }
    Ok(grid_map)
}

pub fn load_gridmap_from_tilemap_manual<T: IdentifiableTile, B: IdentTileBuilder<T>>(
    tilemap: &Gd<TileMap>,
    collection: &GodotTileCollection,
    builder: &B,
) -> Result<GridMap2D<T>, GodotTileError> {
    let size = size_from_rect(tilemap.get_used_rect());

    let mut grid_map = GridMap2D::new(size);
    let mut sources = InfoBuilder::default();

    for coord in tilemap.get_used_cells(0).iter_shared() {
        let source_id = tilemap.get_cell_source_id(0, coord);

        let info = sources.build(source_id, tilemap, coord);

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
    gridmap: GridMap2D<T>,
    tilemap: &mut Gd<TileMap>,
    collection: &GodotTileCollection,
) -> Result<(), GodotTileError> {

    for position in gridmap.get_all_positions() {
        let tile = gridmap
            .get_tile_at_position(&position)
            .expect("cannot get tile!");

        if let Some(godot_info) = collection.get_tile_data(tile.tile_type_id()) {
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

#[derive(Default)]
struct InfoBuilder {
    inner: HashMap<i32, TileSourceType>,
}

impl InfoBuilder {
    fn identify(&mut self, source_id: i32, tilemap: &Gd<TileMap>) -> TileSourceType {
        match self.inner.entry(source_id) {
            Entry::Vacant(e) => {
                let source = tilemap
                    .get_tileset()
                    .expect("no tileset added!")
                    .get_source(source_id)
                    .unwrap_or_else(|| panic!("no tileset source with ID: {source_id}"));

                let tile_source = if let Err(obj) = source.try_cast::<TileSetAtlasSource>() {
                    if obj.try_cast::<TileSetScenesCollectionSource>().is_ok() {
                        TileSourceType::Collection
                    } else {
                        panic!("cannot cast source with id: {source_id} to either ScenesCollection or Atlas")
                    }
                } else {
                    TileSourceType::Atlas
                };
                *e.insert(tile_source)
            }
            Entry::Occupied(e) => *e.get(),
        }
    }

    fn build(&mut self, source_id: i32, tilemap: &Gd<TileMap>, coord: Vector2i) -> GodotTileInfo {
        match self.identify(source_id, tilemap) {
            TileSourceType::Atlas => GodotTileInfo::new_atlas(
                source_id,
                tilemap.get_cell_atlas_coords(0, coord),
                tilemap.get_cell_alternative_tile(0, coord),
            ),
            TileSourceType::Collection => {
                GodotTileInfo::new_scene(source_id, tilemap.get_cell_alternative_tile(0, coord))
            }
            TileSourceType::Mesh => unreachable!(),
        }
    }
}

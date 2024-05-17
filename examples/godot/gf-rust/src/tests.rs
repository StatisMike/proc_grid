use gd_rehearse::itest::gditest;
use godot::{
    builtin::Rect2i,
    engine::{load, TileMap, TileSet},
    obj::{Gd, NewAlloc},
};
use grid_forge::{
    godot::godot::ops::{load_gridmap_from_tilemap_manual, write_gridmap_to_tilemap},
    map::GridSize,
    tile::identifiable::{
        builders::IdentTileTraitBuilder, BasicIdentTileData, IdentifiableTile, IdentifiableTileData,
    },
};

use crate::tile_collections::TileCollections;

const TILESET_RESOURCE_PATH: &str = "res://tileset.tres";
const SOURCE_ID: i32 = 0;
const TILESET_IMG_PATH: &str = "res://tiles/all_tiles.png";

const ROADS_MAP_PATH: &str = "res://tiles/roads.png";
const SEAS_MAP_PATH: &str = "res://tiles/seas.png";

#[gditest]
fn test_load_into_gridmap() {
    let collection = get_test_collection();
    let cloned = collection.clone();
    let binding = collection.bind();
    let tileset = load::<TileSet>(TILESET_RESOURCE_PATH);

    let roads_map = binding.load_vis_map_from_path(ROADS_MAP_PATH).unwrap();
    let mut roads_tilemap = TileMap::new_alloc();
    roads_tilemap.set_tileset(tileset.clone());
    write_gridmap_to_tilemap(
        &roads_map,
        &mut roads_tilemap,
        binding.godot_collection.as_ref().unwrap(),
    )
    .expect("cannot write roads to tilemap");

    let seas_map = binding.load_vis_map_from_path(SEAS_MAP_PATH).unwrap();
    let mut seas_tilemap = TileMap::new_alloc();
    seas_tilemap.set_tileset(tileset);
    write_gridmap_to_tilemap(
        &seas_map,
        &mut seas_tilemap,
        binding.godot_collection.as_ref().unwrap(),
    )
    .expect("cannot write seas to tilemap");

    assert_eq!(
        roads_map.size().x(),
        size_from_rect(roads_tilemap.get_used_rect()).x()
    );
    assert_eq!(
        roads_map.size().y(),
        size_from_rect(roads_tilemap.get_used_rect()).y()
    );
    assert_eq!(
        seas_map.size().x(),
        size_from_rect(seas_tilemap.get_used_rect()).x()
    );
    assert_eq!(
        seas_map.size().y(),
        size_from_rect(seas_tilemap.get_used_rect()).y()
    );

    std::mem::drop(binding);

    roads_tilemap.free();
    seas_tilemap.free();
    cloned.free()
}

#[gditest]
fn test_from_grindmap_identical() {
    let collection = get_test_collection();
    let cloned = collection.clone();
    let binding = collection.bind();
    let builder = IdentTileTraitBuilder::<BasicIdentTileData>::default();
    let tileset = load::<TileSet>(TILESET_RESOURCE_PATH);

    let roads_map = binding.load_vis_map_from_path(ROADS_MAP_PATH).unwrap();
    let mut roads_tilemap = TileMap::new_alloc();
    roads_tilemap.set_tileset(tileset.clone());
    write_gridmap_to_tilemap(
        &roads_map,
        &mut roads_tilemap,
        binding.godot_collection.as_ref().unwrap(),
    )
    .expect("cannot write roads to tilemap");

    let seas_map = binding.load_vis_map_from_path(SEAS_MAP_PATH).unwrap();
    let mut seas_tilemap = TileMap::new_alloc();
    seas_tilemap.set_tileset(tileset);
    write_gridmap_to_tilemap(
        &seas_map,
        &mut seas_tilemap,
        binding.godot_collection.as_ref().unwrap(),
    )
    .expect("cannot write seas to tilemap");

    let second_roads = load_gridmap_from_tilemap_manual(
        &roads_tilemap,
        binding.godot_collection.as_ref().unwrap(),
        &builder,
    )
    .expect("cannot load roads to gridmap");
    let second_seas = load_gridmap_from_tilemap_manual(
        &seas_tilemap,
        binding.godot_collection.as_ref().unwrap(),
        &builder,
    )
    .expect("cannot load roads to gridmap");

    for position in roads_map.get_all_positions() {
        assert_eq!(
            roads_map
                .get_tile_at_position(&position)
                .unwrap()
                .tile_type_id(),
            second_roads
                .get_tile_at_position(&position)
                .unwrap()
                .tile_type_id()
        );
    }

    for position in seas_map.get_all_positions() {
        assert_eq!(
            seas_map
                .get_tile_at_position(&position)
                .unwrap()
                .tile_type_id(),
            second_seas
                .get_tile_at_position(&position)
                .unwrap()
                .tile_type_id()
        );
    }

    std::mem::drop(binding);

    roads_tilemap.free();
    seas_tilemap.free();
    cloned.free();
}

fn get_test_collection() -> Gd<TileCollections> {
    let mut collection = TileCollections::new_alloc();
    let out = collection.clone();

    let tileset = load::<TileSet>(TILESET_RESOURCE_PATH);

    let mut bind = collection.bind_mut();
    bind.set_path_to_image(TILESET_IMG_PATH.into());
    bind.set_source_id(SOURCE_ID);
    bind.set_tileset(Some(tileset));
    bind.generate_collections();

    out
}

fn size_from_rect(rect: Rect2i) -> GridSize {
    GridSize::new_xy(
        (rect.size.x - rect.position.x) as u32,
        (rect.size.y - rect.position.y) as u32,
    )
}

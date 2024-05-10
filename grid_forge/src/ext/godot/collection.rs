use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use godot::{
    builtin::{Vector2i, Vector3i},
    engine::{GridMap, TileMap, TileSet, TileSetAtlasSource, TileSetScenesCollectionSource},
    obj::Gd,
};

use crate::tile::identifiable::collection::IdentTileCollection;

#[derive(Clone, Copy, Hash, Debug)]
pub struct GodotAtlasTileInfo {
    gd_source_id: i32,
    gd_atlas_coord: Vector2i,
    gd_alternative_id: i32,
}

#[derive(Clone, Copy, Hash, Debug)]
pub struct GodotScenesCollectionInfo {
    gd_source_id: i32,
    gd_tile_idx: i32,
}

#[derive(Clone, Copy, Hash, Debug)]
pub struct GodotMeshItemData {
    gd_tile_idx: i32,
}

#[derive(Clone, Copy, Debug, Hash)]
pub enum GodotTileInfo {
    Atlas(GodotAtlasTileInfo),
    ScenesCollection(GodotScenesCollectionInfo),
    MeshLibrary(GodotMeshItemData),
}

impl GodotTileInfo {
    pub fn new_atlas(gd_source_id: i32, gd_atlas_coord: Vector2i, gd_alternative_id: i32) -> Self {
        Self::Atlas(GodotAtlasTileInfo {
            gd_source_id,
            gd_atlas_coord,
            gd_alternative_id,
        })
    }

    pub fn new_scene(gd_source_id: i32, gd_tile_idx: i32) -> Self {
        Self::ScenesCollection(GodotScenesCollectionInfo {
            gd_source_id,
            gd_tile_idx,
        })
    }

    pub fn new_mesh(gd_tile_idx: i32) -> Self {
        Self::MeshLibrary(GodotMeshItemData { gd_tile_idx })
    }

    pub fn get_tile_type_id(&self) -> u64 {
        let mut hasher = DefaultHasher::default();

        self.hash(&mut hasher);

        hasher.finish()
    }

    pub fn generate_gd_type_id_atlas(
        source_id: i32,
        atlas_coord: Vector2i,
        alternative_id: i32,
    ) -> u64 {
        let mut hasher = DefaultHasher::default();

        source_id.hash(&mut hasher);
        atlas_coord.hash(&mut hasher);
        alternative_id.hash(&mut hasher);

        hasher.finish()
    }

    pub fn insert_to_tilemap(&self, tilemap: &mut Gd<TileMap>, coords: Vector2i, layer: i32) {
        match &self {
            GodotTileInfo::Atlas(tile_info) => tilemap
                .set_cell_ex(layer, coords)
                .source_id(tile_info.gd_source_id)
                .atlas_coords(tile_info.gd_atlas_coord)
                .alternative_tile(tile_info.gd_alternative_id)
                .done(),
            GodotTileInfo::ScenesCollection(tile_info) => tilemap
                .set_cell_ex(layer, coords)
                .source_id(tile_info.gd_source_id)
                .alternative_tile(tile_info.gd_tile_idx)
                .done(),
            GodotTileInfo::MeshLibrary(_) => panic!("cannot add MeshLibrary tile to TileMap"),
        }
    }

    pub fn insert_to_gridmap(&self, gridmap: &mut Gd<GridMap>, coords: Vector3i) {
        match &self {
            GodotTileInfo::MeshLibrary(tile_info) => gridmap
                .set_cell_item_ex(coords, tile_info.gd_tile_idx)
                .done(),
            _ => panic!("only MeshLibrary tile can be added to GridMap"),
        }
    }
}

pub struct GodotTileCollection {
    inner: HashMap<u64, GodotTileInfo>,
    rev: HashMap<u64, u64>,
}

impl GodotTileCollection {
    pub fn load_tiles_from_tileset(&mut self, tileset: &Gd<TileSet>) {
        for source_idx in 0..tileset.get_source_count() {
            let tileset_id = tileset.get_source_id(source_idx);
            let tileset_source = tileset
                .get_source(tileset_id)
                .unwrap_or_else(|| panic!("cannot get atlas source with ID: {tileset_id}"));

            match tileset_source.try_cast::<TileSetAtlasSource>() {
                Ok(atlas) => self.load_source_atlas(atlas, tileset_id),
                Err(source) => {
                    if let Ok(collection) = source.try_cast::<TileSetScenesCollectionSource>() {
                        self.load_source_scenes(collection, tileset_id);
                    }
                }
            }
        }
    }

    pub fn load_source_atlas(&mut self, source: Gd<TileSetAtlasSource>, source_id: i32) {
        for tile_idx in 0..source.get_tiles_count() {
            let atlas_coord = source.get_tile_id(tile_idx);
            let alternative_id = 0;
            let tile_type_id =
                GodotTileInfo::generate_gd_type_id_atlas(source_id, atlas_coord, alternative_id);
            self.inner.insert(
                tile_type_id,
                GodotTileInfo::new_atlas(source_id, atlas_coord, alternative_id),
            );
        }
    }

    pub fn load_source_scenes(
        &mut self,
        source: Gd<TileSetScenesCollectionSource>,
        source_id: i32,
    ) {
        // Wrong header in Godot method
        let mut source = source.clone();
        for tile_idx in 0..source.get_scene_tiles_count() {
            let tile_id = source.get_scene_tile_id(tile_idx);
            let tile_type_id =
                GodotTileInfo::generate_gd_type_id_atlas(source_id, Vector2i::default(), tile_id);
            self.inner
                .insert(tile_type_id, GodotTileInfo::new_scene(source_id, tile_id));
        }
    }
}

impl IdentTileCollection for GodotTileCollection {
    type DATA = GodotTileInfo;

    fn inner(&self) -> &HashMap<u64, Self::DATA> {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut HashMap<u64, Self::DATA> {
        &mut self.inner
    }

    fn rev(&self) -> &HashMap<u64, u64> {
        &self.rev
    }

    fn rev_mut(&mut self) -> &mut HashMap<u64, u64> {
        &mut self.rev
    }
}

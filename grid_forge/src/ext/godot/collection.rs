use std::collections::hash_map::{Entry, HashMap};
use std::hash::{DefaultHasher, Hash, Hasher};

use godot::builtin::{Vector2i, Vector3i};
use godot::engine::{GridMap, TileMap, TileSet, TileSetAtlasSource, TileSetScenesCollectionSource};
use godot::obj::Gd;

use crate::tile::identifiable::collection::IdentTileCollection;

use super::TileSourceType;

#[derive(Default)]
pub struct GodotInfoBuilder {
    inner: HashMap<i32, TileSourceType>,
}

impl GodotInfoBuilder {
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

    pub fn build_from_tilemap(
        &mut self,
        source_id: i32,
        tilemap: &Gd<TileMap>,
        coord: Vector2i,
    ) -> GodotTileMapTileInfo {
        match self.identify(source_id, tilemap) {
            TileSourceType::Atlas => GodotTileMapTileInfo::new_atlas(
                source_id,
                tilemap.get_cell_atlas_coords(0, coord),
                tilemap.get_cell_alternative_tile(0, coord),
            ),
            TileSourceType::Collection => GodotTileMapTileInfo::new_scene(
                source_id,
                tilemap.get_cell_alternative_tile(0, coord),
            ),
            TileSourceType::Mesh => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, Hash, Debug)]
pub struct GodotAtlasTileInfo {
    pub gd_source_id: i32,
    pub gd_atlas_coord: Vector2i,
    pub gd_alternative_id: i32,
}

#[derive(Clone, Copy, Hash, Debug)]
pub struct GodotScenesCollectionInfo {
    pub gd_source_id: i32,
    pub gd_tile_idx: i32,
}

#[derive(Clone, Copy, Hash, Debug)]
pub enum GodotGridMapTileInfo {
    MeshLibrary(GodotMeshItemData),
}
#[derive(Clone, Copy, Hash, Debug)]
pub struct GodotMeshItemData {
    pub gd_tile_idx: i32,
}

impl GodotGridMapTileInfo {
    pub fn new_mesh(gd_tile_idx: i32) -> Self {
        Self::MeshLibrary(GodotMeshItemData { gd_tile_idx })
    }

    pub fn insert_to_gridmap(&self, gridmap: &mut Gd<GridMap>, coords: Vector3i) {
        match &self {
            Self::MeshLibrary(tile_info) => gridmap
                .set_cell_item_ex(coords, tile_info.gd_tile_idx)
                .done(),
        }
    }
}

#[derive(Clone, Copy, Debug, Hash)]
pub enum GodotTileMapTileInfo {
    Atlas(GodotAtlasTileInfo),
    ScenesCollection(GodotScenesCollectionInfo),
}

impl GodotTileMapTileInfo {
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
            GodotTileMapTileInfo::Atlas(tile_info) => tilemap
                .set_cell_ex(layer, coords)
                .source_id(tile_info.gd_source_id)
                .atlas_coords(tile_info.gd_atlas_coord)
                .alternative_tile(tile_info.gd_alternative_id)
                .done(),
            GodotTileMapTileInfo::ScenesCollection(tile_info) => tilemap
                .set_cell_ex(layer, coords)
                .source_id(tile_info.gd_source_id)
                .alternative_tile(tile_info.gd_tile_idx)
                .done(),
        }
    }
}

#[derive(Default, Clone)]
pub struct GodotTileMapCollection {
    inner: HashMap<u64, GodotTileMapTileInfo>,
    rev: HashMap<u64, u64>,
}

impl GodotTileMapCollection {
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
                GodotTileMapTileInfo::new_atlas(source_id, atlas_coord, alternative_id)
                    .get_tile_type_id();
            self.inner.insert(
                tile_type_id,
                GodotTileMapTileInfo::new_atlas(source_id, atlas_coord, alternative_id),
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
                GodotTileMapTileInfo::new_scene(source_id, tile_id).get_tile_type_id();
            self.inner.insert(
                tile_type_id,
                GodotTileMapTileInfo::new_scene(source_id, tile_id),
            );
        }
    }
}

impl IdentTileCollection for GodotTileMapCollection {
    type DATA = GodotTileMapTileInfo;

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

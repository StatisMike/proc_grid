//! Implements synchronized loading of `*.png` representation of tiles into `grid_forge` on the Rust side and
//! [`TileSet`] on the Godot side.
//!
//! Parallel loading is made to make sure that the `tile_type_id` of [`BasicIdentTileData`] matches between
//! [`VisCollection`] and [`GodotTileMapCollection`].

use std::collections::HashSet;

use godot::builtin::{Array, Color, GString, Vector2i};
use godot::engine::{AcceptDialog, Image, Texture2D, TileMap, TileSet, TileSetAtlasSource};
use godot::log::{godot_error, godot_warn};
use godot::obj::Gd;
use godot::register::{godot_api, GodotClass};

use grid_forge::godot::collection::{GodotTileMapCollection, GodotTileMapTileInfo};
use grid_forge::godot::ops::write_gridmap_to_tilemap;
use grid_forge::map::GridMap2D;
use grid_forge::tile::identifiable::builders::IdentTileTraitBuilder;
use grid_forge::tile::identifiable::collection::IdentTileCollection;
use grid_forge::tile::identifiable::BasicIdentTileData;
use grid_forge::vis::collection::VisCollection;
use grid_forge::vis::ops::{check_grid_vis_size, load_gridmap_identifiable_manual};
use grid_forge::vis::{read_tile, DefaultVisPixel, PixelWithDefault};

use image::Rgb;

/// TileCollections holds reference to chosen GodotTileset and and png tileset, gathering their information
/// synchronously to create analogous [`VisCollection`] and [`GodotTileMapCollection`]
#[derive(GodotClass)]
#[class(base=Node, init)]
pub struct TileCollections {
    /// Necessary for VisCollection
    #[export(file = "*.png")]
    path_to_image: GString,
    /// Necessary for GodotCollection
    #[export]
    tileset: Option<Gd<TileSet>>,
    #[export]
    source_id: i32,

    #[var]
    tiles: Array<Gd<SingleTile>>,

    /// Messages
    #[export]
    modal: Option<Gd<AcceptDialog>>,

    #[var]
    generated: bool,

    pub vis_collection: Option<VisCollection<DefaultVisPixel, 4, 4>>,
    pub godot_collection: Option<GodotTileMapCollection>,
}

#[godot_api]
impl TileCollections {
    #[func]
    /// Main method, handling initialization of the tile collections.
    /// It needs to generate both collections side-by-side, for the `tile_type_id` to be synchronized.
    pub fn generate_collections(&mut self) {
        // Field checks
        if self.tileset.is_none() {
            godot_error!("no tileset has been assigned in `TileCollections");
            return;
        }

        let atlas = self
            .tileset
            .clone()
            .unwrap()
            .get_source(self.source_id)
            .and_then(|source| source.try_cast::<TileSetAtlasSource>().ok());
        if atlas.is_none() {
            godot_error!(
                "cannot cast tileset source with `source_id` {} into `TileSetAtlasSource`",
                self.source_id
            );
            return;
        }

        // Open image file
        let image = image_buffer_from_texture_2d(self.path_to_image.clone());

        let grid_size = check_grid_vis_size::<DefaultVisPixel, 4, 4>(&image);

        if let Err(error) = grid_size {
            godot_error!("{}", error);
            return;
        }
        let grid_size = grid_size.unwrap();

        // Began gathering Collections simultanously to make sure that both type tile ids match up.
        let mut vis_collection: VisCollection<image::Rgb<u8>, 4, 4> = VisCollection::default();
        vis_collection.set_empty_tile_pixels(Some([[DefaultVisPixel::pix_default(); 4]; 4]));
        let mut godot_collection = GodotTileMapCollection::default();

        for (tile_type_id, position) in grid_size.get_all_possible_positions().iter().enumerate() {
            let mut pixels = [[DefaultVisPixel::pix_default(); 4]; 4];
            if let Err(error) = read_tile(&mut pixels, &image, position) {
                godot_error!("{error}");
                return;
            }

            let gd_image = SingleTile::pixels_to_image(&pixels);
            let types = SingleTile::generate_types(&pixels);
            vis_collection.add_tile_pixels_manual(tile_type_id as u64, pixels);

            let godot_tile_info =
                GodotTileMapTileInfo::new_atlas(self.source_id, position.get_godot_coords(), 0);

            self.tiles.push(SingleTile::new(
                tile_type_id as i32,
                godot_tile_info,
                gd_image,
                types,
            ));
            godot_collection.add_tile_data(tile_type_id as u64, godot_tile_info);
        }

        self.vis_collection = Some(vis_collection);
        self.godot_collection = Some(godot_collection);

        self.generated = true;
    }

    #[func]
    /// Converts png to tilemap, showing modal dialog with error if any.
    fn convert_png_to_tilemap(&self, path: GString, tilemap: Gd<TileMap>) {
        let mut tilemap = tilemap.clone();

        let map = match self.load_vis_map_from_path(&path.clone().to_string()) {
            Ok(map) => map,
            Err(message) => {
                self.show_modal(&message);
                return;
            }
        };

        if let Err(message) = self.grid_map_into_tilemap(&map, &mut tilemap) {
            self.show_modal(&message);
        };
    }

    fn show_modal(&self, message: &str) {
        if let Some(modal) = &self.modal {
            let mut pntr = modal.clone();
            pntr.set_text(message.into());
            pntr.set_visible(true);
        } else {
            godot_warn!("Cannot find modal for TileCollections. Message to show: {message}");
        }
    }

    /// Loads GridMap from PNG representation using its VisCollection.
    pub fn load_vis_map_from_path(
        &self,
        path: &str,
    ) -> Result<GridMap2D<BasicIdentTileData>, String> {
        let image = image_buffer_from_texture_2d(path);
        let vis_collection = self
            .vis_collection
            .as_ref()
            .ok_or("Cannot get VisCollection")?;
        let builder = IdentTileTraitBuilder::<BasicIdentTileData>::default();

        load_gridmap_identifiable_manual(&image, vis_collection, &builder)
            .map_err(|err| format!("Image to GridMap conversion failed: {err}"))
    }

    /// Loads tiles from GridMap into Godot TileMap using its GodotTileMapCollection.
    pub fn grid_map_into_tilemap(
        &self,
        map: &GridMap2D<BasicIdentTileData>,
        tilemap: &mut Gd<TileMap>,
    ) -> Result<(), String> {
        write_gridmap_to_tilemap(
            map,
            tilemap,
            self.godot_collection
                .as_ref()
                .ok_or("Cannot get GodotCollection".to_string())?,
        )
        .map_err(|err| format!("GridMap to TileMap conversion failed: {err}"))?;

        Ok(())
    }

    #[func]
    pub fn insert_tile(&self, tilemap: Gd<TileMap>, tile_type_id: u64, coords: Vector2i) {
        let mut tilemap = tilemap.clone();
        self.godot_collection
            .as_ref()
            .unwrap()
            .get_tile_data(&tile_type_id)
            .unwrap()
            .insert_to_tilemap(&mut tilemap, coords, 0)
    }
}

#[derive(GodotClass)]
#[class(base=Resource, no_init)]
pub struct SingleTile {
    #[var]
    tile_type_id: i32,
    info: GodotTileMapTileInfo,
    #[var]
    image: Gd<Image>,
    types: Vec<u32>,
}

#[godot_api]
impl SingleTile {
    #[constant]
    pub const SAND: u32 = 0;

    #[constant]
    pub const WATER: u32 = 1;

    #[constant]
    pub const GRASS: u32 = 2;

    #[constant]
    pub const ROAD: u32 = 3;

    fn new(
        tile_type_id: i32,
        info: GodotTileMapTileInfo,
        image: Gd<Image>,
        types: Vec<u32>,
    ) -> Gd<Self> {
        Gd::from_object(Self {
            tile_type_id,
            info,
            image,
            types,
        })
    }

    #[func]
    pub fn get_atlas_coords(&self) -> Vector2i {
        let GodotTileMapTileInfo::Atlas(tile_info) = self.info else {
            godot_error!("Cannot get atlas coords from tile info");
            return Vector2i::ZERO;
        };
        tile_info.gd_atlas_coord
    }

    #[func]
    pub fn has_type(&self, type_id: u32) -> bool {
        self.types.contains(&type_id)
    }

    #[func]
    pub fn insert_into(&self, mut into: Gd<TileMap>, coords: Vector2i) {
        self.info.insert_to_tilemap(&mut into, coords, 0);
    }
}

impl SingleTile {
    fn generate_types(pixels: &[[DefaultVisPixel; 4]; 4]) -> Vec<u32> {
        let mut types = HashSet::new();
        for row in pixels.iter() {
            for pixel in row.iter() {
                if let Some(type_id) = Self::pix_classify(pixel) {
                    types.insert(type_id);
                }
            }
        }
        types.into_iter().collect()
    }

    fn pix_classify(pixel: &DefaultVisPixel) -> Option<u32> {
        match pixel {
            Rgb([64, 158, 24]) => Some(Self::GRASS),
            Rgb([133, 79, 40]) => Some(Self::ROAD),
            Rgb([222, 230, 9]) => Some(Self::SAND),
            Rgb([53, 60, 209]) => Some(Self::WATER),
            _ => None,
        }
    }

    fn pixels_to_image(pixels: &[[DefaultVisPixel; 4]; 4]) -> Gd<Image> {
        let mut image = Image::create(4, 4, false, godot::engine::image::Format::RGB8).unwrap();
        for (y, row) in pixels.iter().enumerate() {
            for (x, pixel) in row.iter().enumerate() {
                let color = Color::from_rgba8(pixel.0[0], pixel.0[1], pixel.0[2], 0);
                image.set_pixel(x as i32, y as i32, color);
            }
        }
        image
    }
}

/// Utility function to load `image::ImageBuffer` from Godot's `Texture2D` resource path.
fn image_buffer_from_texture_2d(
    texture_path: impl Into<GString>,
) -> image::ImageBuffer<image::Rgb<u8>, Vec<u8>> {
    let texture = godot::prelude::load::<Texture2D>(texture_path);
    let image_data = texture.get_image().unwrap().get_data().to_vec();

    image::ImageBuffer::from_raw(
        texture.get_width() as u32,
        texture.get_height() as u32,
        image_data,
    )
    .unwrap()
}

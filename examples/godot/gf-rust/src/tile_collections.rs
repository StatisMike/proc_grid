use std::io::BufReader;

use godot::{
    builtin::{GString, Vector2i},
    engine::{
        file_access::ModeFlags, AcceptDialog, FileAccess, GFile, GridMap, Popup, Resource, TileMap, TileSet, TileSetAtlasSource
    },
    log::{godot_error, godot_print, godot_warn},
    obj::{Base, Gd},
    register::{godot_api, GodotClass},
};

use grid_forge::{
    godot::godot::{
        collection::{GodotTileMapCollection, GodotTileMapTileInfo},
        ops::write_gridmap_to_tilemap,
    },
    map::GridMap2D,
    tile::{
        identifiable::{
            builders::IdentTileTraitBuilder, collection::IdentTileCollection,
            BasicIdentifiableTile2D,
        },
        vis::{DefaultVisPixel, PixelWithDefault},
    },
    vis::{
        collection::VisCollection,
        ops::{check_grid_vis_size, load_gridmap_identifiable_manual},
        read_tile,
    },
};
use image::ImageFormat;

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

    /// Messages
    #[export]
    modal: Option<Gd<AcceptDialog>>,

    #[var]
    generated: bool,

    pub vis_collection: Option<VisCollection<BasicIdentifiableTile2D, DefaultVisPixel, 4, 4>>,
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

        let gd_file = FileAccess::open(self.path_to_image.clone(), ModeFlags::READ);
        if gd_file.is_none() {
            godot_error!(
                "cannot open image file at specified Godot location: {}",
                self.path_to_image
            );
            return;
        };
        let path = gd_file.unwrap().get_path_absolute().to_string();

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
        let image = image::open(&path);
        if image.is_err() {
            godot_error!("cannot open image file at absolute location: {path}");
            return;
        }
        let image = image.unwrap().to_rgb8();

        let grid_size = check_grid_vis_size::<DefaultVisPixel, 4, 4>(&image);

        if let Err(error) = grid_size {
            godot_error!("{}", error);
            return;
        }
        let grid_size = grid_size.unwrap();

        // Began gathering Collections simultanously to make sure that both type tile ids match up.
        let mut vis_collection: VisCollection<BasicIdentifiableTile2D, image::Rgb<u8>, 4, 4> =
            VisCollection::default();
        vis_collection.set_empty_tile_pixels(Some([[DefaultVisPixel::pix_default(); 4]; 4]));
        let mut godot_collection = GodotTileMapCollection::default();

        for (tile_type_id, position) in grid_size.get_all_possible_positions().iter().enumerate() {
            let mut pixels = [[DefaultVisPixel::pix_default(); 4]; 4];
            if let Err(error) = read_tile(&mut pixels, &image, position) {
                godot_error!("{error}");
                return;
            }
            vis_collection.add_tile_pixels_manual(tile_type_id as u64, pixels);

            let godot_tile_info = GodotTileMapTileInfo::new_atlas(
                self.source_id,
                Vector2i {
                    x: position.0 as i32,
                    y: position.1 as i32,
                },
                0,
            );
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
          },
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
    ) -> Result<GridMap2D<BasicIdentifiableTile2D>, String> {
        let gfile =
            GFile::open(path, ModeFlags::READ).map_err(|err| format!("File open error: {err}"))?;
        let reader = BufReader::new(gfile);
        let image = image::load(reader, ImageFormat::Png)
            .map_err(|err| format!("Image read error: {err}"))?
            .into_rgb8();

        let vis_collection = self
            .vis_collection
            .as_ref()
            .ok_or("Cannot get VisCollection")?;
        let builder = IdentTileTraitBuilder::<BasicIdentifiableTile2D>::default();

        load_gridmap_identifiable_manual(&image, vis_collection, &builder)
            .map_err(|err| format!("Image to GridMap conversion failed: {err}"))
    }

    /// Loads tiles from GridMap into Godot TileMap using its GodotTileMapCollection.
    pub fn grid_map_into_tilemap(
        &self,
        map: &GridMap2D<BasicIdentifiableTile2D>,
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
}

#![allow(dead_code)]

use grid_forge::{
    gen::collapse::CollapsedTileData,
    map::GridMap2D,
    tile::identifiable::builders::IdentTileTraitBuilder,
    vis::{collection::VisCollection, ops::load_gridmap_identifiable_auto, DefaultVisPixel},
};
use image::{ImageBuffer, Rgb};
use rand::SeedableRng;
use rand_chacha::ChaChaRng;

#[derive(Debug)]
pub struct RngHelper {
    seed: [u8; 32],
    pos: Option<u128>,
}

impl RngHelper {
    pub fn init_str(phrase: &str, fill: u8) -> Self {
        let mut seed: [u8; 32] = [fill; 32];

        for (i, byte) in phrase.as_bytes().iter().enumerate() {
            if i < 32 {
                seed[i] = *byte
            }
        }

        Self { seed, pos: None }
    }

    pub fn with_pos(mut self, pos: u128) -> Self {
        self.pos = Some(pos);
        self
    }

    pub fn print_state(rng: &ChaChaRng) {
        println!(
            "Seed: {:?}; Pos: {}, Stream: {}",
            rng.get_seed(),
            rng.get_word_pos(),
            rng.get_stream()
        )
    }
}

impl From<RngHelper> for ChaChaRng {
    fn from(value: RngHelper) -> ChaChaRng {
        let mut rng = rand_chacha::ChaChaRng::from_seed(value.seed);

        if let Some(pos) = value.pos {
            rng.set_word_pos(pos);
        }

        rng
    }
}

pub enum VisRotate {
    None,
    R90,
    R180,
    R270,
}

impl VisRotate {
    pub fn rotate(
        &self,
        buffer: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    ) -> Option<ImageBuffer<Rgb<u8>, Vec<u8>>> {
        match self {
            VisRotate::None => None,
            VisRotate::R90 => Some(image::imageops::rotate90(buffer)),
            VisRotate::R180 => Some(image::imageops::rotate180(buffer)),
            VisRotate::R270 => Some(image::imageops::rotate270(buffer)),
        }
    }
}

pub struct VisGridLoaderHelper<'a> {
    collection: &'a mut VisCollection<DefaultVisPixel, 4, 4>,
}

impl<'a> VisGridLoaderHelper<'a> {
    pub fn new(collection: &'a mut VisCollection<DefaultVisPixel, 4, 4>) -> Self {
        Self { collection }
    }

    pub fn load_w_rotate(
        &mut self,
        paths: &[&str],
        rotations: &[VisRotate],
    ) -> Vec<GridMap2D<CollapsedTileData>> {
        let mut out = Vec::new();
        let builder = IdentTileTraitBuilder::default();
        for path in paths {
            let image = self.load_image_grid(path);

            for rotation in rotations {
                out.push(if let Some(rotated) = rotation.rotate(&image) {
                    load_gridmap_identifiable_auto(&rotated, self.collection, &builder).unwrap()
                } else {
                    load_gridmap_identifiable_auto(&image, self.collection, &builder).unwrap()
                });
            }
        }
        out
    }

    fn load_image_grid(&self, path: &str) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        image::open(path).unwrap().into_rgb8()
    }
}

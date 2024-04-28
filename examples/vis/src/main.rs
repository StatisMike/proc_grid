// This example shows general implementation of the `vis` feature, which allows generating image out of created `GridMap`.
// This is very useful in development state, as before creating maps out of final desired GridTile it is best to test
// out the algorithms used, but is rarely useful in final build.

// Most examples use the `vis` feature to present visual representation of GridMap2D.

use grid_forge::{
    map::{GridMap2D, GridSize}, tile::{
        vis::{DefaultVisPixel, VisTile2D},
        GridTile2D,
    }, vis::ops::{init_map_image_buffer, write_gridmap_vis}, GridPos2D
};
use image::imageops;
use rand::{Rng, SeedableRng};

// Enum holding the easily discernable colors for the resulting tiles.
enum TileColor {
    Blue,
    Green,
}

impl TileColor {
    fn rgb(&self) -> DefaultVisPixel {
        match self {
            TileColor::Blue => DefaultVisPixel::from([52, 119, 235]),
            TileColor::Green => DefaultVisPixel::from([128, 235, 52]),
        }
    }
}

// GridTile struct besides required GridPos2D holds also the created enum.
struct TwoColoredTile {
    pos: GridPos2D,
    color: TileColor,
}

impl GridTile2D for TwoColoredTile {
    fn grid_position(&self) -> GridPos2D {
        self.pos
    }

    fn set_grid_position(&mut self, position: GridPos2D) {
        self.pos = position;
    }
}

// Trait necessary
impl VisTile2D<DefaultVisPixel, 1, 1> for TwoColoredTile {
    fn vis_pixels(&self) -> [[DefaultVisPixel; 1]; 1] {
        [[self.color.rgb()]]
    }
}

fn main() {
    // Seed for reproductability.
    let mut seed: [u8; 32] = [0; 32];

    for (i, byte) in "vis_example".as_bytes().iter().enumerate() {
        if i < 31 {
            seed[i] = *byte;
        }
    }
    let mut rng = rand_chacha::ChaChaRng::from_seed(seed);

    // Create an empty GridMap...
    let size = GridSize::new(100, 100);
    let mut map = GridMap2D::<TwoColoredTile>::new(size);

    // and fill it with colors at random.
    for pos in map.size().get_all_possible_positions() {
        let color = if rng.gen_bool(0.5) {
            TileColor::Blue
        } else {
            TileColor::Green
        };
        let tile = TwoColoredTile { pos, color };
        map.insert_tile(tile);
    }

    // Create image and save it in examples dir.
    let mut image = init_map_image_buffer::<DefaultVisPixel, 1, 1>(&size);
    write_gridmap_vis(&mut image, &map).unwrap();
    let image = imageops::resize(
        &image,
        map.size().x() * 5,
        map.size().y() * 5,
        imageops::FilterType::Nearest,
    );
    image.save("examples/vis_example.png").unwrap();
}

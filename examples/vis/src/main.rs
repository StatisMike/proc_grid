// This example shows general implementation of the `vis` feature, which allows generating image out of created `GridMap`.
// This is very useful in development state, as before creating maps out of final desired GridTile it is best to test
// out the algorithms used, but is rarely useful in final build.

// Most examples use the `vis` feature.

use image::Rgb;
use rand::{Rng, SeedableRng};

use grid_forge::prelude::*;
use grid_forge::vis::*;

// Enum holding the easily discernable colors for the resulting tiles.
enum TileColor {
    Blue,
    Green,
}

impl TileColor {
    fn rgb(&self) -> Rgb<u8> {
        match self {
            TileColor::Blue => Rgb::<u8>::from([52, 119, 235]),
            TileColor::Green => Rgb::<u8>::from([128, 235, 52]),
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
impl VisTile2D<Rgb<u8>> for TwoColoredTile {
    const PIXEL_SIZE: [u32; 2] = [8, 8];

    fn vis_pixel(&self) -> Rgb<u8> {
        self.color.rgb()
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
    let mut map = GridMap2D::<TwoColoredTile>::new(GridSize::new(100, 100));

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
    let image = map.vis_grid_map();
    image.save("examples/vis_example.jpg").unwrap();
}

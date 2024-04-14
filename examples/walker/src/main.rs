use grid_forge::gen::walker::GridWalker2DBuilder;
use grid_forge::prelude::*;
use grid_forge::vis::*;
use image::Rgb;
use rand::SeedableRng;

#[derive(Clone)]
enum TileColor {
    Black,
    Red,
}

impl TileColor {
    fn rgb(&self) -> Rgb<u8> {
        match self {
            TileColor::Black => Rgb::<u8>::from([0, 0, 0]),
            TileColor::Red => Rgb::<u8>::from([235, 32, 32]),
        }
    }
}

#[derive(Clone)]
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

impl VisTile2D<Rgb<u8>> for TwoColoredTile {
    const PIXEL_SIZE: [u32; 2] = [8, 8];

    fn vis_pixel(&self) -> Rgb<u8> {
        self.color.rgb()
    }
}

fn main() {
    // Seed for reproductability.
    let mut seed: [u8; 32] = [1; 32];

    for (i, byte) in "walker_example".as_bytes().iter().enumerate() {
        if i < 31 {
            seed[i] = *byte;
        }
    }
    let rng = rand_chacha::ChaChaRng::from_seed(seed);

    let size = GridSize::new(255, 255);
    let mut walker = GridWalker2DBuilder::default()
        .with_size(size)
        .with_current_pos(size.center())
        .with_rng(rng)
        .with_max_step_size(5)
        .build()
        .unwrap();

    while walker.current_iters() <= 45000 {
        walker.walk();
    }

    let mut map = GridMap2D::new(size);

    for pos in walker.walked() {
        let tile = TwoColoredTile {
            pos: *pos,
            color: TileColor::Red,
        };
        map.insert_tile(tile);
    }

    map.fill_empty_with(TwoColoredTile {
        pos: (0, 0),
        color: TileColor::Black,
    });

    let image = map.vis_grid_map();

    image.save("examples/walker.jpg").unwrap();
}

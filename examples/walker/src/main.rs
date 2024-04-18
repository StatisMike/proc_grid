use grid_forge::{
    gen::walker::GridWalker2DBuilder,
    map::{GridSize, vis::VisGrid2D},
    tile::{vis::VisTile2D, GridTile2D},
    GridPos2D,
};
use image::Rgb;
use rand::SeedableRng;

#[derive(Clone, Hash)]
enum TileColor {
    Gray,
    Red,
}

impl TileColor {
    fn rgb(&self) -> Rgb<u8> {
        match self {
            TileColor::Gray => Rgb::<u8>::from([32, 32, 32]),
            TileColor::Red => Rgb::<u8>::from([235, 32, 32]),
        }
    }
}

#[derive(Clone, Hash)]
struct TwoColoredTile {
    pos: GridPos2D,
    color: TileColor,
}

impl TwoColoredTile {
    fn new(pos: GridPos2D, color: TileColor) -> Self {
        Self { pos, color }
    }
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

    fn vis_pixels(&self) -> Rgb<u8> {
        self.color.rgb()
    }
}

fn main() {
    // Seed for reproductability.
    let mut seed: [u8; 32] = [0; 32];

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
        .with_min_step_size(2)
        .with_max_step_size(5)
        .build()
        .unwrap();

    while walker.current_iters() <= 45000 {
        walker.walk();
    }

    let mut map = walker.gen_grid_map(|pos| TwoColoredTile::new(pos, TileColor::Red));

    map.fill_empty_using(|pos| TwoColoredTile::new(pos, TileColor::Gray));

    let image = map.vis_grid_map();

    image.save("examples/walker_example.jpg").unwrap();
}

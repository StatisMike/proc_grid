//! Shows the usage of Random Walker generative algorithm.

use grid_forge::gen::walker::GridWalker2DBuilder;
use grid_forge::vis::ops::{init_map_image_buffer, write_gridmap_vis};
use grid_forge::vis::{DefaultVisPixel, VisTileData};
use grid_forge::{GridPosition, GridSize, GridTile, TileData};

use image::imageops::resize;
use rand::SeedableRng;

#[derive(Clone, Hash)]
enum TileColor {
    Gray,
    Red,
}

impl TileColor {
    fn rgb(&self) -> DefaultVisPixel {
        match self {
            TileColor::Gray => DefaultVisPixel::from([32, 32, 32]),
            TileColor::Red => DefaultVisPixel::from([235, 32, 32]),
        }
    }
}

#[derive(Clone, Hash)]
struct TwoColoredTileData {
    color: TileColor,
}

impl TwoColoredTileData {
    fn new(color: TileColor) -> Self {
        Self { color }
    }
}

impl TileData for TwoColoredTileData {}

impl VisTileData<DefaultVisPixel, 1, 1> for TwoColoredTileData {
    fn vis_pixels(&self) -> [[DefaultVisPixel; 1]; 1] {
        [[self.color.rgb()]]
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

    let size = GridSize::new_xy(255, 255);
    let mut walker = GridWalker2DBuilder::default()
        .with_size(size)
        .with_current_pos(GridPosition::new_xy(size.center().0, size.center().1))
        .with_rng(rng)
        .with_min_step_size(2)
        .with_max_step_size(5)
        .build()
        .unwrap();

    while walker.current_iters() <= 45000 {
        walker.walk();
    }

    let mut map =
        walker.gen_grid_map(|pos| GridTile::new(pos, TwoColoredTileData::new(TileColor::Red)));
    map.fill_empty_using(|pos| GridTile::new(pos, TwoColoredTileData::new(TileColor::Gray)));

    let mut image = init_map_image_buffer::<DefaultVisPixel, 1, 1>(&size);

    write_gridmap_vis(&mut image, &map).unwrap();

    let image = resize(
        &image,
        map.size().x() * 5,
        map.size().y() * 5,
        image::imageops::FilterType::Nearest,
    );

    image
        .save(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/outputs/walker_example.png"
        ))
        .unwrap();
}

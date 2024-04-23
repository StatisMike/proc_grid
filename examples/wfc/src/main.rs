use std::fs::rename;

use grid_forge::{
    gen::wfc::{
        analyzer::WFCAnalyzer,
        builder::WFCCloneBuilder,
        resolver::WFCResolver,
        vis::{WFCVisGrid2D, WFCVisTile},
        WFCTile,
    },
    map::{vis::VisGrid2D, GridMap2D, GridSize},
    tile::{vis::DefaultVisPixel, GridTile2D},
};
use image::{imageops, load, open, ImageBuffer, Pixel};
use rand::SeedableRng;

type MyTile = WFCVisTile<DefaultVisPixel, 4, 4>;
type MyGrid = GridMap2D<MyTile>;

fn main() {
    // Initialize builder, which will take care of putting new tiles on specific places.
    // As `WFCVisTile` is basic and implements `Clone`, the `WFCCloneBuilder` can be used. In other scenarios, the
    // `WFCFunBuilder` needs to be used.
    let mut builder = WFCCloneBuilder::default();

    // Load samples as grid maps.
    // let seas_img = open("../assets/samples/seas.png").unwrap().into_rgb8();
    let seas_img = open("examples/assets/samples/seas.png")
        .unwrap()
        .into_rgb8();
    let seas_grid = MyGrid::from_image(&seas_img).unwrap();
    // Add tiles to the builder.
    builder.add_tiles(seas_grid.iter_tiles(), false);

    // let roads_img = open("../assets/samples/roads.png").unwrap().into_rgb8();
    let roads_img = open("examples/assets/samples/roads.png")
        .unwrap()
        .into_rgb8();
    let roads_grid = MyGrid::from_image(&roads_img).unwrap();
    // Add tiles to the builder.
    builder.add_tiles(roads_grid.iter_tiles(), false);

    // Construct WFC Analyzer and provide the maps for analyzing.
    let mut analyzer = WFCAnalyzer::new();
    analyzer.analyze(&seas_grid);
    analyzer.analyze(&roads_grid);

    // Seed for reproductability.
    // let mut seed: [u8; 32] = [6; 32];
    let mut seed: [u8; 32] = [142; 32];

    for (i, byte) in "wfc_example".as_bytes().iter().enumerate() {
        if i < 31 {
            seed[i] = *byte;
        }
    }

    let mut rng = rand_chacha::ChaChaRng::from_seed(seed);

    // Create new grid via WFC Resolver.
    let size = GridSize::new(20, 20);
    let mut resolver = WFCResolver::new(size, &analyzer);
    resolver.populate_map_all(&mut rng);

    let mut can_process = true;

    while can_process {
        can_process = resolver.process(&mut rng);
    }

    println!(
        "resolved: {} / {}",
        resolver.n_resolved(),
        size.get_all_possible_positions().len()
    );

    let new_map = resolver.build_grid(&builder);

    new_map.vis_grid_map().save("examples/wfc.png").unwrap()
}

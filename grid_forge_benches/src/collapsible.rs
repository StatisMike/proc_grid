extern crate test;

use grid_forge::{
    gen::collapse::*,
    map::GridSize,
    tile::{
        identifiable::{builders::IdentTileTraitBuilder, BasicIdentTileData},
        vis::DefaultVisPixel,
    },
    vis::{collection::VisCollection, ops::load_gridmap_identifiable_auto},
};
use rand::SeedableRng;
use rand_chacha::ChaChaRng;
use test::Bencher;

#[bench]
fn gen_identity_position_10x10(bencher: &mut Bencher) {
    let builder = IdentTileTraitBuilder::<BasicIdentTileData>::default();
    let mut collection = VisCollection::<DefaultVisPixel, 4, 4>::default();

    let seas_img = image::open("../assets/samples/seas.png")
        .unwrap()
        .into_rgb8();

    let seas_grid = load_gridmap_identifiable_auto(&seas_img, &mut collection, &builder).unwrap();

    let roads_img = image::open("../assets/samples/roads.png")
        .unwrap()
        .into_rgb8();
    let roads_grid = load_gridmap_identifiable_auto(&roads_img, &mut collection, &builder).unwrap();

    let mut analyzer = AdjacencyIdentityAnalyzer::default();
    analyzer.analyze(&seas_grid);
    analyzer.analyze(&roads_grid);

    let mut frequency_hints = FrequencyHints::default();
    frequency_hints.analyze_grid_map(&seas_grid);
    frequency_hints.analyze_grid_map(&roads_grid);

    let size = GridSize::new_xy(10, 10);

    bencher.iter(|| {
        // Seed for reproductability
        let mut seed = [4u8; 32];
        let bytes = "i am benchmarking".as_bytes();
        seed[..bytes.len().min(32)].copy_from_slice(bytes);

        let mut rng = ChaChaRng::from_seed(seed);

        let mut resolver = CollapsibleResolver::new(size);
        resolver
            .generate(
                &mut rng,
                &size.get_all_possible_positions(),
                PositionQueue::default(),
                &frequency_hints,
                analyzer.adjacency(),
            )
            .unwrap();
    });
}

#[bench]
fn gen_identity_entrophy_10x10(bencher: &mut Bencher) {
    let builder = IdentTileTraitBuilder::<BasicIdentTileData>::default();
    let mut collection = VisCollection::<DefaultVisPixel, 4, 4>::default();

    let seas_img = image::open("../assets/samples/seas.png")
        .unwrap()
        .into_rgb8();

    let seas_grid = load_gridmap_identifiable_auto(&seas_img, &mut collection, &builder).unwrap();

    let roads_img = image::open("../assets/samples/roads.png")
        .unwrap()
        .into_rgb8();
    let roads_grid = load_gridmap_identifiable_auto(&roads_img, &mut collection, &builder).unwrap();

    let mut analyzer = AdjacencyIdentityAnalyzer::default();
    analyzer.analyze(&seas_grid);
    analyzer.analyze(&roads_grid);

    let mut frequency_hints = FrequencyHints::default();
    frequency_hints.analyze_grid_map(&seas_grid);
    frequency_hints.analyze_grid_map(&roads_grid);

    let size = GridSize::new_xy(10, 10);

    bencher.iter(|| {
        // Seed for reproductability
        let mut seed = [0u8; 32];
        let bytes = "i am benchmarking".as_bytes();
        seed[..bytes.len().min(32)].copy_from_slice(bytes);

        let mut rng = ChaChaRng::from_seed(seed);

        let mut resolver = CollapsibleResolver::new(size);
        resolver
            .generate(
                &mut rng,
                &size.get_all_possible_positions(),
                EntrophyQueue::default(),
                &frequency_hints,
                analyzer.adjacency(),
            )
            .unwrap();
    });
}

#[bench]
fn gen_border_position_10x10(bencher: &mut Bencher) {
    let builder = IdentTileTraitBuilder::<BasicIdentTileData>::default();
    let mut collection = VisCollection::<DefaultVisPixel, 4, 4>::default();

    let seas_img = image::open("../assets/samples/seas.png")
        .unwrap()
        .into_rgb8();

    let seas_grid = load_gridmap_identifiable_auto(&seas_img, &mut collection, &builder).unwrap();

    let roads_img = image::open("../assets/samples/roads.png")
        .unwrap()
        .into_rgb8();
    let roads_grid = load_gridmap_identifiable_auto(&roads_img, &mut collection, &builder).unwrap();

    let mut analyzer = AdjacencyBorderAnalyzer::default();
    analyzer.analyze(&seas_grid);
    analyzer.analyze(&roads_grid);

    let mut frequency_hints = FrequencyHints::default();
    frequency_hints.analyze_grid_map(&seas_grid);
    frequency_hints.analyze_grid_map(&roads_grid);

    let size = GridSize::new_xy(10, 10);

    bencher.iter(|| {
        // Seed for reproductability
        let mut seed = [0u8; 32];
        let bytes = "collapse_gen_example".as_bytes();
        seed[..bytes.len().min(32)].copy_from_slice(bytes);

        let mut rng = ChaChaRng::from_seed(seed);

        let mut resolver = CollapsibleResolver::new(size);
        resolver
            .generate(
                &mut rng,
                &size.get_all_possible_positions(),
                PositionQueue::default(),
                &frequency_hints,
                analyzer.adjacency(),
            )
            .unwrap();
    });
}

#[bench]
fn gen_border_entrophy_10x10(bencher: &mut Bencher) {
    let builder = IdentTileTraitBuilder::<BasicIdentTileData>::default();
    let mut collection = VisCollection::<DefaultVisPixel, 4, 4>::default();

    let seas_img = image::open("../assets/samples/seas.png")
        .unwrap()
        .into_rgb8();

    let seas_grid = load_gridmap_identifiable_auto(&seas_img, &mut collection, &builder).unwrap();

    let roads_img = image::open("../assets/samples/roads.png")
        .unwrap()
        .into_rgb8();
    let roads_grid = load_gridmap_identifiable_auto(&roads_img, &mut collection, &builder).unwrap();

    let mut analyzer = AdjacencyBorderAnalyzer::default();
    analyzer.analyze(&seas_grid);
    analyzer.analyze(&roads_grid);

    let mut frequency_hints = FrequencyHints::default();
    frequency_hints.analyze_grid_map(&seas_grid);
    frequency_hints.analyze_grid_map(&roads_grid);

    let size = GridSize::new_xy(10, 10);

    bencher.iter(|| {
        // Seed for reproductability
        let mut seed = [0u8; 32];
        let bytes = "collapse_gen_example".as_bytes();
        seed[..bytes.len().min(32)].copy_from_slice(bytes);

        let mut rng = ChaChaRng::from_seed(seed);

        let mut resolver = CollapsibleResolver::new(size);
        resolver
            .generate(
                &mut rng,
                &size.get_all_possible_positions(),
                EntrophyQueue::default(),
                &frequency_hints,
                analyzer.adjacency(),
            )
            .unwrap();
    });
}

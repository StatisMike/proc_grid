extern crate test;

use grid_forge::{
    gen::collapse::singular::*,
    gen::collapse::*,
    map::GridSize,
    tile::identifiable::{builders::IdentTileTraitBuilder, BasicIdentTileData},
    vis::{collection::VisCollection, ops::load_gridmap_identifiable_auto, DefaultVisPixel},
};
use rand::SeedableRng;
use rand_chacha::ChaChaRng;
use test::Bencher;

use crate::utils::RngHelper;

#[bench]
fn analyze_adjacency_identity_10x10(bencher: &mut Bencher) {
    let builder = IdentTileTraitBuilder::<BasicIdentTileData>::default();
    let mut collection = VisCollection::<DefaultVisPixel, 4, 4>::default();

    let seas_img = image::open("../assets/samples/seas.png")
        .unwrap()
        .into_rgb8();

    let seas_grid = load_gridmap_identifiable_auto(&seas_img, &mut collection, &builder).unwrap();

    bencher.iter(|| {
        let mut analyzer = IdentityAnalyzer::default();
        analyzer.analyze(&seas_grid);
    });
}

#[bench]
fn analyze_adjacency_border_10x10(bencher: &mut Bencher) {
    let builder = IdentTileTraitBuilder::<BasicIdentTileData>::default();
    let mut collection = VisCollection::<DefaultVisPixel, 4, 4>::default();

    let seas_img = image::open("../assets/samples/seas.png")
        .unwrap()
        .into_rgb8();

    let seas_grid = load_gridmap_identifiable_auto(&seas_img, &mut collection, &builder).unwrap();

    bencher.iter(|| {
        let mut analyzer = BorderAnalyzer::default();
        analyzer.analyze(&seas_grid);
    });
}

#[bench]
fn analyze_frequency_10x10(bencher: &mut Bencher) {
    let builder = IdentTileTraitBuilder::<BasicIdentTileData>::default();
    let mut collection = VisCollection::<DefaultVisPixel, 4, 4>::default();

    let seas_img = image::open("../assets/samples/seas.png")
        .unwrap()
        .into_rgb8();

    let seas_grid = load_gridmap_identifiable_auto(&seas_img, &mut collection, &builder).unwrap();

    bencher.iter(|| {
        let mut freq_hints = FrequencyHints::default();
        freq_hints.analyze_grid_map(&seas_grid);
    })
}

#[bench]
fn analyze_build_collapsible_grid(bencher: &mut Bencher) {
    let builder = IdentTileTraitBuilder::<BasicIdentTileData>::default();
    let mut collection = VisCollection::<DefaultVisPixel, 4, 4>::default();

    let seas_img = image::open("../assets/samples/seas.png")
        .unwrap()
        .into_rgb8();

    let seas_grid = load_gridmap_identifiable_auto(&seas_img, &mut collection, &builder).unwrap();

    let mut analyzer = BorderAnalyzer::default();
    analyzer.analyze(&seas_grid);
    let adj_rules = analyzer.adjacency();
    let mut freq_hints = FrequencyHints::default();
    freq_hints.analyze_grid_map(&seas_grid);

    bencher.iter(|| {
        let _grid =
            CollapsibleTileGrid::new_empty(GridSize::new_xy(10, 10), &freq_hints, adj_rules);
    })
}

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

    let mut analyzer = IdentityAnalyzer::default();
    analyzer.analyze(&seas_grid);
    analyzer.analyze(&roads_grid);

    let mut frequency_hints = FrequencyHints::default();
    frequency_hints.analyze_grid_map(&seas_grid);
    frequency_hints.analyze_grid_map(&roads_grid);

    let size = GridSize::new_xy(10, 10);
    let mut grid = CollapsibleTileGrid::new_empty(size, &frequency_hints, analyzer.adjacency());

    bencher.iter(|| {
        // Seed for reproductability
        let mut rng: ChaChaRng = RngHelper::init_str("singular_identity", 0)
            .with_pos(1008)
            .into();

        let mut resolver = Resolver::default();
        resolver
            .generate(
                &mut grid,
                &mut rng,
                &size.get_all_possible_positions(),
                PositionQueue::default(),
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

    let mut analyzer = IdentityAnalyzer::default();
    analyzer.analyze(&seas_grid);
    analyzer.analyze(&roads_grid);

    let mut frequency_hints = FrequencyHints::default();
    frequency_hints.analyze_grid_map(&seas_grid);
    frequency_hints.analyze_grid_map(&roads_grid);

    let size = GridSize::new_xy(10, 10);
    let mut grid = CollapsibleTileGrid::new_empty(size, &frequency_hints, analyzer.adjacency());

    bencher.iter(|| {
        // Seed for reproductability
        let mut seed = [0u8; 32];
        let bytes = "i am benchmarking".as_bytes();
        seed[..bytes.len().min(32)].copy_from_slice(bytes);

        let mut rng = ChaChaRng::from_seed(seed);

        let mut resolver = Resolver::default();
        resolver
            .generate(
                &mut grid,
                &mut rng,
                &size.get_all_possible_positions(),
                EntrophyQueue::default(),
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

    let mut analyzer = BorderAnalyzer::default();
    analyzer.analyze(&seas_grid);
    analyzer.analyze(&roads_grid);

    let mut frequency_hints = FrequencyHints::default();
    frequency_hints.analyze_grid_map(&seas_grid);
    frequency_hints.analyze_grid_map(&roads_grid);

    let size = GridSize::new_xy(10, 10);
    let mut grid = CollapsibleTileGrid::new_empty(size, &frequency_hints, analyzer.adjacency());

    bencher.iter(|| {
        // Seed for reproductability
        let mut rng: ChaChaRng = RngHelper::init_str("singular_border", 0)
            .with_pos(354)
            .into();

        let mut resolver = Resolver::default();
        resolver
            .generate(
                &mut grid,
                &mut rng,
                &size.get_all_possible_positions(),
                PositionQueue::default(),
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

    let mut analyzer = BorderAnalyzer::default();
    analyzer.analyze(&seas_grid);
    analyzer.analyze(&roads_grid);

    let mut frequency_hints = FrequencyHints::default();
    frequency_hints.analyze_grid_map(&seas_grid);
    frequency_hints.analyze_grid_map(&roads_grid);

    let size = GridSize::new_xy(10, 10);
    let mut grid = CollapsibleTileGrid::new_empty(size, &frequency_hints, analyzer.adjacency());

    bencher.iter(|| {
        // Seed for reproductability
        let mut seed = [0u8; 32];
        let bytes = "collapse_gen_example".as_bytes();
        seed[..bytes.len().min(32)].copy_from_slice(bytes);

        let mut rng = ChaChaRng::from_seed(seed);

        let mut resolver = Resolver::default();
        resolver
            .generate(
                &mut grid,
                &mut rng,
                &size.get_all_possible_positions(),
                EntrophyQueue::default(),
            )
            .unwrap();
    });
}

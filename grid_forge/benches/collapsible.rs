#![cfg(feature = "vis")]
#![cfg(feature = "gen")]

#[macro_use]
extern crate bencher;

use bencher::Bencher;
use grid_forge::{
    gen::{
        adjacency::AdjacencyAnalyzer, collapse::{frequency::FrequencyHints, queue::{EntrophyQueue, PositionQueue}, resolver::CollapsibleResolver}, ms::MSAnalyzer, wfc::analyzer::WFCAnalyzer
    },
    map::GridSize,
    tile::{
        identifiable::{builder::IdentTileTraitBuilder, BasicIdentifiableTile2D},
        vis::DefaultVisPixel,
    },
    vis::{collection::VisCollection, ops::load_gridmap_identifiable_auto},
};
use rand::SeedableRng;
use rand_chacha::ChaChaRng;

fn gen_wfc_10x10(bencher: &mut Bencher) {
    let builder = IdentTileTraitBuilder::<BasicIdentifiableTile2D>::default();
    let mut collection = VisCollection::<BasicIdentifiableTile2D, DefaultVisPixel, 4, 4>::default();

    let seas_img = image::open("../assets/samples/seas.png")
        .unwrap()
        .into_rgb8();

    let seas_grid = load_gridmap_identifiable_auto(&seas_img, &mut collection, &builder).unwrap();

    let roads_img = image::open("../assets/samples/roads.png")
        .unwrap()
        .into_rgb8();
    let roads_grid = load_gridmap_identifiable_auto(&roads_img, &mut collection, &builder).unwrap();

    let mut analyzer: WFCAnalyzer<BasicIdentifiableTile2D> = WFCAnalyzer::default();
    analyzer.analyze(&seas_grid);
    analyzer.analyze(&roads_grid);

    let mut frequency_hints = FrequencyHints::default();
    frequency_hints.analyze_grid_map(&seas_grid);
    frequency_hints.analyze_grid_map(&roads_grid);

    let size = GridSize::new(10, 10);

    bencher.iter(|| {
            // Seed for reproductability
    let mut seed = [0u8; 32];
    let bytes = "i am benchmarking".as_bytes();
    seed[..bytes.len().min(32)].copy_from_slice(bytes);

    let mut rng = ChaChaRng::from_seed(seed);

        let mut resolver = CollapsibleResolver::new(size);
        resolver.generate(
            &mut rng,
            &size.get_all_possible_positions(), 
            EntrophyQueue::default(), 
            &frequency_hints, 
            analyzer.adjacency()).unwrap();

    });
}

fn gen_ms_10x10(bencher: &mut Bencher) {
    let builder = IdentTileTraitBuilder::<BasicIdentifiableTile2D>::default();
    let mut collection = VisCollection::<BasicIdentifiableTile2D, DefaultVisPixel, 4, 4>::default();

    let seas_img = image::open("../assets/samples/seas.png")
        .unwrap()
        .into_rgb8();

    let seas_grid = load_gridmap_identifiable_auto(&seas_img, &mut collection, &builder).unwrap();

    let roads_img = image::open("../assets/samples/roads.png")
        .unwrap()
        .into_rgb8();
    let roads_grid = load_gridmap_identifiable_auto(&roads_img, &mut collection, &builder).unwrap();

    let mut analyzer: MSAnalyzer<BasicIdentifiableTile2D> = MSAnalyzer::default();
    analyzer.analyze(&seas_grid);
    analyzer.analyze(&roads_grid);

    let mut frequency_hints = FrequencyHints::default();
    frequency_hints.analyze_grid_map(&seas_grid);
    frequency_hints.analyze_grid_map(&roads_grid);

    let size = GridSize::new(10, 10);

    bencher.iter(|| {
            // Seed for reproductability
    let mut seed = [0u8; 32];
    let bytes = "collapse_gen_example".as_bytes();
    seed[..bytes.len().min(32)].copy_from_slice(bytes);

    let mut rng = ChaChaRng::from_seed(seed);

        let mut resolver = CollapsibleResolver::new(size);
        resolver.generate(
            &mut rng,
            &size.get_all_possible_positions(), 
            PositionQueue::default(), 
            &frequency_hints, 
            analyzer.adjacency()).unwrap();

    });
}

benchmark_group!(benches, gen_wfc_10x10, gen_ms_10x10);
benchmark_main!(benches);

#![cfg(feature = "vis")]
#![cfg(feature = "gen")]

#[macro_use]
extern crate bencher;

use bencher::Bencher;
use grid_forge::{
    gen::{
        adjacency::AdjacencyAnalyzer,
        ms::MSAnalyzer,
        wfc::{analyzer::WFCAnalyzer, resolver::WFCResolver},
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

    let mut analyzer = WFCAnalyzer::default();
    analyzer.analyze(&seas_grid);
    analyzer.analyze(&roads_grid);

    // Seed for reproductability
    let mut seed = [0u8; 32];
    let bytes = "i am benchmarking".as_bytes();
    seed[..bytes.len().min(32)].copy_from_slice(bytes);

    let mut rng = ChaChaRng::from_seed(seed);

    let size = GridSize::new(10, 10);

    bencher.iter(|| {
        let mut resolver = WFCResolver::new(size, &analyzer);
        resolver.populate_map_all(&mut rng);

        let mut can_process = true;

        while can_process {
            can_process = resolver.process(&mut rng);
        }

        assert_eq!(100, resolver.n_resolved());
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

    let mut analyzer = MSAnalyzer::default();
    analyzer.analyze(&seas_grid);
    analyzer.analyze(&roads_grid);

    // Seed for reproductability
    let mut seed = [0u8; 32];
    let bytes = "i am benchmarking".as_bytes();
    seed[..bytes.len().min(32)].copy_from_slice(bytes);

    let mut rng = ChaChaRng::from_seed(seed);

    let size = GridSize::new(10, 10);

    bencher.iter(|| {
        let mut resolver = WFCResolver::new(size, &analyzer);
        resolver.populate_map_all(&mut rng);

        let mut can_process = true;

        while can_process {
            can_process = resolver.process(&mut rng);
        }

        assert_eq!(100, resolver.n_resolved());
    });
}

benchmark_group!(benches, gen_wfc_10x10, gen_ms_10x10);
benchmark_main!(benches);

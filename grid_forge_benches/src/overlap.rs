extern crate test;

use rand_chacha::ChaChaRng;
use test::Bencher;

use grid_forge::{
    gen::collapse::overlap::*,
    gen::collapse::*,
    map::GridSize,
    tile::identifiable::{builders::IdentTileTraitBuilder, BasicIdentTileData},
    vis::{collection::VisCollection, ops::load_gridmap_identifiable_auto, DefaultVisPixel},
};

use crate::utils::RngHelper;

const MAP_10X10: &str = "../assets/samples/seas.png";
const MAP_20X20: &str = "../assets/samples/roads.png";

#[bench]
fn analyze_10x10_pattern_2x2(bencher: &mut Bencher) {
    let builder = IdentTileTraitBuilder::<BasicIdentTileData>::default();
    let mut collection = VisCollection::<DefaultVisPixel, 4, 4>::default();

    let img = image::open(MAP_10X10).unwrap().into_rgb8();

    let grid = load_gridmap_identifiable_auto(&img, &mut collection, &builder).unwrap();

    bencher.iter(|| {
        let mut analyzer = Analyzer::<OverlappingPattern2D<2, 2>, BasicIdentTileData>::default();
        analyzer.analyze_map(&grid);
    })
}

#[bench]
fn analyze_10x10_pattern_3x3(bencher: &mut Bencher) {
    let builder = IdentTileTraitBuilder::<BasicIdentTileData>::default();
    let mut collection = VisCollection::<DefaultVisPixel, 4, 4>::default();

    let img = image::open(MAP_10X10).unwrap().into_rgb8();

    let grid = load_gridmap_identifiable_auto(&img, &mut collection, &builder).unwrap();

    bencher.iter(|| {
        let mut analyzer = Analyzer::<OverlappingPattern2D<3, 3>, BasicIdentTileData>::default();
        analyzer.analyze_map(&grid);
    })
}

#[bench]
fn generate_10x10_pattern_2x2_entrophy(bencher: &mut Bencher) {
    let builder = IdentTileTraitBuilder::<BasicIdentTileData>::default();

    let mut vis_collection = VisCollection::<DefaultVisPixel, 4, 4>::default();

    let mut analyzer = Analyzer::<OverlappingPattern2D<2, 2>, BasicIdentTileData>::default();

    for path in &[MAP_10X10, MAP_20X20] {
        let img = image::open(path).unwrap().into_rgb8();

        let grid = load_gridmap_identifiable_auto(&img, &mut vis_collection, &builder).unwrap();

        analyzer.analyze_map(&grid);
    }

    let pattern_collection = analyzer.get_collection().clone();
    let pattern_rules = analyzer.get_adjacency();
    let pattern_freq = analyzer.get_frequency();

    let size = GridSize::new_xy(10, 10);
    let positions = size.get_all_possible_positions();

    let grid =
        CollapsiblePatternGrid::new_empty(size, pattern_collection, &pattern_freq, &pattern_rules)
            .unwrap();

    bencher.iter(|| {
        let mut rng: ChaChaRng = RngHelper::init_str("overlap_bench", 1).into();

        let mut resolver = Resolver::default();
        let res = resolver
            .generate(grid.clone(), &mut rng, &positions, EntrophyQueue::default());

        assert!(res.is_ok());
    });
}

#[bench]
fn generate_10x10_pattern_3x3_entrophy(bencher: &mut Bencher) {
    let builder = IdentTileTraitBuilder::<BasicIdentTileData>::default();

    let mut vis_collection = VisCollection::<DefaultVisPixel, 4, 4>::default();

    let mut analyzer = Analyzer::<OverlappingPattern2D<3, 3>, BasicIdentTileData>::default();

    for path in &[MAP_10X10, MAP_20X20] {
        let img = image::open(path).unwrap().into_rgb8();

        let grid = load_gridmap_identifiable_auto(&img, &mut vis_collection, &builder).unwrap();

        analyzer.analyze_map(&grid);
    }

    let pattern_collection = analyzer.get_collection().clone();
    let pattern_rules = analyzer.get_adjacency();
    let pattern_freq = analyzer.get_frequency();

    let size = GridSize::new_xy(10, 10);
    let positions = size.get_all_possible_positions();

    let grid =
        CollapsiblePatternGrid::new_empty(size, pattern_collection, &pattern_freq, &pattern_rules)
            .unwrap();

    bencher.iter(|| {
        let mut rng: ChaChaRng = RngHelper::init_str("overlap_bench", 1).into();

        let mut resolver = Resolver::default();
        let res = resolver
            .generate(grid.clone(), &mut rng, &positions, EntrophyQueue::default());
        
        assert!(res.is_ok())
            });
}

// #[bench]
// fn generate_10x10_pattern_3x3_entrophy_2(bencher: &mut Bencher) {

//     let builder = IdentTileTraitBuilder::<BasicIdentTileData>::default();

//     let mut vis_collection = VisCollection::<DefaultVisPixel, 4, 4>::default();

//     let mut analyzer = OverlappingAnalyzer::<Pattern2D<3,3>, BasicIdentTileData>::default();

//     for path in &[MAP_10X10, MAP_20X20] {
//         let img = image::open(path)
//         .unwrap()
//         .into_rgb8();

//         let grid = load_gridmap_identifiable_auto(&img, &mut vis_collection, &builder).unwrap();

//         analyzer.analyze_map(&grid);
//     }

//     let pattern_rules = analyzer.get_adjacency();
//     let pattern_collection = analyzer.get_collection();
//     let pattern_freq = analyzer.get_frequency();

//     let size = GridSize::new_xy(10, 10);
//     let positions = size.get_all_possible_positions();

//     bencher.iter(|| {
//         let mut rng: ChaChaRng = RngHelper::init_str("overlap_bench", 1).into();

//         let mut resolver = OverlappingResolver::new(size);
//         resolver.generate(
//             &mut rng,
//             &positions,
//             EntrophyQueue::new(3),
//             pattern_collection,
//             pattern_freq,
//             pattern_rules,
//         ).unwrap();
//     });

// }

extern crate test;

use test::Bencher;

use grid_forge::{
    gen::collapse::{OverlappingAnalyzer, Pattern2D},
    tile::identifiable::{builders::IdentTileTraitBuilder, BasicIdentTileData},
    vis::{collection::VisCollection, ops::load_gridmap_identifiable_auto, DefaultVisPixel},
};

const MAP_10X10: &str = "../assets/samples/seas.png";

#[bench]
fn gen_analyze_10x10_pattern_3x3(bencher: &mut Bencher) {
    let builder = IdentTileTraitBuilder::<BasicIdentTileData>::default();
    let mut collection = VisCollection::<DefaultVisPixel, 4, 4>::default();

    let seas_img = image::open(MAP_10X10).unwrap().into_rgb8();

    let seas_grid = load_gridmap_identifiable_auto(&seas_img, &mut collection, &builder).unwrap();

    // bencher.iter(|| {
    //     let mut analyzer = OverlappingAnalyzer::<_, Pattern2D<3, 3>>::new();
    //     analyzer.analyze_map(&seas_grid);
    // })
}

#[bench]
fn gen_rules_10x10_pattern_3x3(bencher: &mut Bencher) {
    let builder = IdentTileTraitBuilder::<BasicIdentTileData>::default();
    let mut collection = VisCollection::<DefaultVisPixel, 4, 4>::default();

    let seas_img = image::open(MAP_10X10).unwrap().into_rgb8();

    let seas_grid = load_gridmap_identifiable_auto(&seas_img, &mut collection, &builder).unwrap();

    // let mut analyzer = OverlappingAnalyzer::<_, Pattern2D<3, 3>>::new();
    // analyzer.analyze_map(&seas_grid);

    // bencher.iter(|| {
    //     analyzer.generate_pattern_rules();
    // })
}

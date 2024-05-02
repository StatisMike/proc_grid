#![cfg(feature = "vis")]

#[macro_use]
extern crate bencher;

use bencher::Bencher;
use grid_forge::{
    tile::{
        identifiable::{builder::IdentTileTraitBuilder, BasicIdentifiableTile2D},
        vis::DefaultVisPixel,
    },
    vis::{
        collection::VisCollection,
        ops::{
            init_map_image_buffer, load_gridmap_identifiable_auto,
            load_gridmap_identifiable_manual, write_gridmap_identifiable,
        },
    },
};

fn load_gridmap_auto(bencher: &mut Bencher) {
    let builder = IdentTileTraitBuilder::<BasicIdentifiableTile2D>::default();
    let image = image::open("../assets/samples/roads.png")
        .unwrap()
        .into_rgb8();

    bencher.iter(|| {
        let mut collection =
            VisCollection::<BasicIdentifiableTile2D, DefaultVisPixel, 4, 4>::default();
        load_gridmap_identifiable_auto(&image, &mut collection, &builder).unwrap();
    });
}

fn load_gridmap_manual(bencher: &mut Bencher) {
    let builder = IdentTileTraitBuilder::<BasicIdentifiableTile2D>::default();
    let image = image::open("../assets/samples/roads.png")
        .unwrap()
        .into_rgb8();

    let mut collection = VisCollection::<BasicIdentifiableTile2D, DefaultVisPixel, 4, 4>::default();
    load_gridmap_identifiable_auto(&image, &mut collection, &builder).unwrap();

    bencher.iter(|| {
        load_gridmap_identifiable_manual(&image, &collection, &builder).unwrap();
    });
}

fn write_grimap_ident(bencher: &mut Bencher) {
    let builder = IdentTileTraitBuilder::<BasicIdentifiableTile2D>::default();
    let image = image::open("../assets/samples/roads.png")
        .unwrap()
        .into_rgb8();
    let mut collection = VisCollection::<BasicIdentifiableTile2D, DefaultVisPixel, 4, 4>::default();
    let gridmap = load_gridmap_identifiable_auto(&image, &mut collection, &builder).unwrap();

    bencher.iter(|| {
        let mut buffer = init_map_image_buffer::<DefaultVisPixel, 4, 4>(gridmap.size());
        write_gridmap_identifiable(&mut buffer, &gridmap, &collection).unwrap();
    })
}

benchmark_group!(
    benches,
    load_gridmap_auto,
    load_gridmap_manual,
    write_grimap_ident
);
benchmark_main!(benches);

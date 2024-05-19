extern crate test;

use grid_forge::{
    tile::identifiable::{builders::IdentTileTraitBuilder, BasicIdentTileData},
    vis::{
        collection::VisCollection,
        ops::{
            init_map_image_buffer, load_gridmap_identifiable_auto,
            load_gridmap_identifiable_manual, write_gridmap_identifiable,
        },
        DefaultVisPixel,
    },
};
use test::Bencher;

#[bench]
fn load_gridmap_auto(bencher: &mut Bencher) {
    let builder = IdentTileTraitBuilder::<BasicIdentTileData>::default();
    let image = image::open("../assets/samples/roads.png")
        .unwrap()
        .into_rgb8();

    bencher.iter(|| {
        let mut collection = VisCollection::<DefaultVisPixel, 4, 4>::default();
        load_gridmap_identifiable_auto(&image, &mut collection, &builder).unwrap();
    });
}

#[bench]
fn load_gridmap_manual(bencher: &mut Bencher) {
    let builder = IdentTileTraitBuilder::<BasicIdentTileData>::default();
    let image = image::open("../assets/samples/roads.png")
        .unwrap()
        .into_rgb8();

    let mut collection = VisCollection::<DefaultVisPixel, 4, 4>::default();
    load_gridmap_identifiable_auto(&image, &mut collection, &builder).unwrap();

    bencher.iter(|| {
        load_gridmap_identifiable_manual(&image, &collection, &builder).unwrap();
    });
}

#[bench]
fn write_grimap_ident(bencher: &mut Bencher) {
    let builder = IdentTileTraitBuilder::<BasicIdentTileData>::default();
    let image = image::open("../assets/samples/roads.png")
        .unwrap()
        .into_rgb8();
    let mut collection = VisCollection::<DefaultVisPixel, 4, 4>::default();
    let gridmap = load_gridmap_identifiable_auto(&image, &mut collection, &builder).unwrap();

    bencher.iter(|| {
        let mut buffer = init_map_image_buffer::<DefaultVisPixel, 4, 4>(gridmap.size());
        write_gridmap_identifiable(&mut buffer, &gridmap, &collection).unwrap();
    })
}

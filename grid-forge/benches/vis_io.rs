use criterion::{criterion_group, criterion_main, Criterion};

use grid_forge::{
    identifiable::{builders::IdentTileTraitBuilder, BasicIdentTileData},
    vis::{
        collection::VisCollection,
        ops::{
            init_map_image_buffer, load_gridmap_identifiable_auto,
            load_gridmap_identifiable_manual, write_gridmap_identifiable,
        },
        DefaultVisPixel,
    },
};

fn load_gridmap_auto(c: &mut Criterion) {
    let builder = IdentTileTraitBuilder::<BasicIdentTileData>::default();
    let image = image::open("../assets/samples/roads.png")
        .unwrap()
        .into_rgb8();

    c.bench_function("load_gridmap_auto", |b| {
        b.iter(|| {
            let mut collection = VisCollection::<DefaultVisPixel, 4, 4>::default();
            load_gridmap_identifiable_auto(&image, &mut collection, &builder).unwrap();
        });
    });
}

fn load_gridmap_manual(c: &mut Criterion) {
    let builder = IdentTileTraitBuilder::<BasicIdentTileData>::default();
    let image = image::open("../assets/samples/roads.png")
        .unwrap()
        .into_rgb8();

    let mut collection = VisCollection::<DefaultVisPixel, 4, 4>::default();
    load_gridmap_identifiable_auto(&image, &mut collection, &builder).unwrap();

    c.bench_function("load_gridmap_manual", |b| {
        b.iter(|| {
            load_gridmap_identifiable_manual(&image, &collection, &builder).unwrap();
        });
    });
}

fn write_gridmap_ident(c: &mut Criterion) {
    let builder = IdentTileTraitBuilder::<BasicIdentTileData>::default();
    let image = image::open("../assets/samples/roads.png")
        .unwrap()
        .into_rgb8();
    let mut collection = VisCollection::<DefaultVisPixel, 4, 4>::default();
    let gridmap = load_gridmap_identifiable_auto(&image, &mut collection, &builder).unwrap();

    c.bench_function("write_gridmap_ident", |b| {
        b.iter(|| {
            let mut buffer = init_map_image_buffer::<DefaultVisPixel, 4, 4>(gridmap.size());
            write_gridmap_identifiable(&mut buffer, &gridmap, &collection).unwrap();
        });
    });
}

criterion_group!(
    benches,
    load_gridmap_auto,
    load_gridmap_manual,
    write_gridmap_ident
);
criterion_main!(benches);

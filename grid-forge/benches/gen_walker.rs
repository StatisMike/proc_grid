mod utils;

use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};

use grid_forge::{
    gen::walker::GridWalker2DBuilder,
    {GridSize, GridTile, TileData},
};
use rand_chacha::ChaCha20Rng;
use utils::RngHelper;

struct EmptyTileData;
impl TileData for EmptyTileData {}

fn walker_walk_4500(c: &mut Criterion) {
    let grid_size = GridSize::new_xy(255, 255);

    c.bench_function("walker_walk_4500", |b| {
        b.iter(|| {
            let rng: ChaCha20Rng = RngHelper::init_str("walker", 0).into();

            let mut walker = GridWalker2DBuilder::default()
                .with_size(grid_size)
                .with_rng(rng)
                .with_min_step_size(2)
                .with_max_step_size(5)
                .build()
                .unwrap();

            while walker.current_iters() <= 4500 {
                walker.walk();
            }
        });
    });
}

fn walker_walk_45000(c: &mut Criterion) {
    let grid_size = GridSize::new_xy(255, 255);

    c.bench_function("walker_walk_45000", |b| {
        b.iter(|| {
            let rng: ChaCha20Rng = RngHelper::init_str("walker", 0).into();

            let mut walker = GridWalker2DBuilder::default()
                .with_size(grid_size)
                .with_rng(rng)
                .with_min_step_size(2)
                .with_max_step_size(5)
                .build()
                .unwrap();

            while walker.current_iters() <= 45000 {
                walker.walk();
            }
        });
    });
}

fn walker_grid_4500(c: &mut Criterion) {
    let grid_size = GridSize::new_xy(255, 255);

    let rng: ChaCha20Rng = RngHelper::init_str("walker", 0).into();

    let mut walker = GridWalker2DBuilder::default()
        .with_size(grid_size)
        .with_rng(rng)
        .with_min_step_size(2)
        .with_max_step_size(5)
        .build()
        .unwrap();

    while walker.current_iters() <= 4500 {
        walker.walk();
    }

    c.bench_function("walker_grid_4500", |b| {
        b.iter(|| {
            walker.gen_grid_map(|pos| GridTile::new(pos, EmptyTileData));
        });
    });
}

fn walker_grid_45000(c: &mut Criterion) {
    let grid_size = GridSize::new_xy(255, 255);

    let rng: ChaCha20Rng = RngHelper::init_str("walker", 0).into();

    let mut walker = GridWalker2DBuilder::default()
        .with_size(grid_size)
        .with_rng(rng)
        .with_min_step_size(2)
        .with_max_step_size(5)
        .build()
        .unwrap();

    while walker.current_iters() <= 45000 {
        walker.walk();
    }

    c.bench_function("walker_grid_45000", |b| {
        b.iter(|| {
            walker.gen_grid_map(|pos| GridTile::new(pos, EmptyTileData));
        });
    });
}

criterion_group! {
  name = benches;
  config = Criterion::default().measurement_time(Duration::from_secs(10));
  targets = walker_walk_4500, walker_walk_45000, walker_grid_4500, walker_grid_45000
}
criterion_main!(benches);

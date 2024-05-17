extern crate test;

use grid_forge::{
    gen::walker::GridWalker2DBuilder,
    map::GridSize,
    tile::{GridPosition, GridTile, TileData},
};
use rand::thread_rng;
use test::Bencher;

struct EmptyTileData;
impl TileData for EmptyTileData {}

#[bench]
fn walker_walk_4500(bench: &mut Bencher) {
    let grid_size = GridSize::new_xy(255, 255);

    let mut walker = GridWalker2DBuilder::default()
        .with_size(grid_size)
        .with_rng(thread_rng())
        .with_min_step_size(2)
        .with_max_step_size(5)
        .build()
        .unwrap();

    bench.iter(|| {
        while walker.current_iters() <= 4500 {
            walker.walk();
        }

        walker.reset();
        walker.set_current_pos(GridPosition::new_xy(
            grid_size.center().0,
            grid_size.center().1,
        ));
    })
}

#[bench]
fn walker_walk_45000(bench: &mut Bencher) {
    let grid_size = GridSize::new_xy(255, 255);

    let mut walker = GridWalker2DBuilder::default()
        .with_size(grid_size)
        .with_rng(thread_rng())
        .with_min_step_size(2)
        .with_max_step_size(5)
        .build()
        .unwrap();

    bench.iter(|| {
        while walker.current_iters() <= 45000 {
            walker.walk();
        }

        walker.reset();
        walker.set_current_pos(GridPosition::new_xy(
            grid_size.center().0,
            grid_size.center().1,
        ));
    })
}

#[bench]
fn walker_grid_4500(bench: &mut Bencher) {
    let grid_size = GridSize::new_xy(255, 255);

    let mut walker = GridWalker2DBuilder::default()
        .with_size(grid_size)
        .with_rng(thread_rng())
        .with_min_step_size(2)
        .with_max_step_size(5)
        .build()
        .unwrap();

    while walker.current_iters() <= 4500 {
        walker.walk();
    }

    bench.iter(|| {
        walker.gen_grid_map(|pos| GridTile::new(pos, EmptyTileData));
    });
}

#[bench]
fn walker_grid_45000(bench: &mut Bencher) {
    let grid_size = GridSize::new_xy(255, 255);

    let mut walker = GridWalker2DBuilder::default()
        .with_size(grid_size)
        .with_rng(thread_rng())
        .with_min_step_size(2)
        .with_max_step_size(5)
        .build()
        .unwrap();

    while walker.current_iters() <= 45000 {
        walker.walk();
    }

    bench.iter(|| {
        walker.gen_grid_map(|pos| GridTile::new(pos, EmptyTileData));
    });
}

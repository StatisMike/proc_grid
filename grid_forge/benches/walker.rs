#![cfg(feature = "gen")]

#[macro_use]
extern crate bencher;

use bencher::Bencher;
use grid_forge::{gen::walker::GridWalker2DBuilder, map::GridSize, tile::GridTile2D, GridPos2D};
use rand::thread_rng;

struct Tile {
    pos: GridPos2D,
}

impl GridTile2D for Tile {
    fn grid_position(&self) -> GridPos2D {
        self.pos
    }
    fn set_grid_position(&mut self, position: GridPos2D) {
        self.pos = position;
    }
}

fn walker_walk_45000(bench: &mut Bencher) {
    let grid_size = GridSize::new(255, 255);

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
        walker.set_current_pos(grid_size.center());
    })
}

fn walker_grid_45000(bench: &mut Bencher) {
    let grid_size = GridSize::new(255, 255);

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
        walker.gen_grid_map(|pos| Tile { pos });
    });
}

benchmark_group!(benches, walker_walk_45000, walker_grid_45000);
benchmark_main!(benches);

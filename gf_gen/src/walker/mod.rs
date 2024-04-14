use std::collections::HashSet;

use gf_defs::{
    error::BuilderError,
    map::{size::GridSize, GridDir, GridMap2D},
    tile::GridTile2D,
    GridPos2D,
};
use rand::Rng;

/// Struct implementing the random walker algorithm, producing the collection of [GridPos2D]. To be created with
/// [GridWalker2DBuilder].
pub struct GridWalker2D<R>
where
    R: Rng,
{
    current_pos: GridPos2D,
    walked: HashSet<GridPos2D>,
    rng: R,
    size: GridSize,
    max_step_size: u8,
    iters: u32,
}

impl<R> GridWalker2D<R>
where
    R: Rng,
{
    pub fn current_iters(&self) -> u32 {
        self.iters
    }

    pub fn walk(&mut self) -> bool {
        self.iters += 1;
        let idx: usize = self.rng.gen_range(0..4);

        let step_size = if self.max_step_size > 1 {
            self.rng.gen_range(1..=self.max_step_size)
        } else {
            1
        };

        for _ in 1..step_size {
            if let Some(pos) = GridDir::ALL[idx].march_step(&self.current_pos, &self.size) {
                self.current_pos = pos;
                self.walked.insert(pos);
            } else {
                return false;
            }
        }
        true
    }

    pub fn walked(&self) -> &HashSet<GridPos2D> {
        &self.walked
    }

    pub fn to_grid_map<T>(&self, tile_fn: fn() -> T) -> GridMap2D<T>
    where
        T: GridTile2D,
    {
        let mut map = GridMap2D::new(self.size);

        for pos in self.walked.iter() {
            let mut tile = tile_fn();
            tile.set_grid_position(*pos);
            map.insert_tile(tile);
        }
        map
    }
}

pub struct GridWalker2DBuilder<R>
where
    R: Rng,
{
    current_pos: Option<GridPos2D>,
    rng: Option<R>,
    size: Option<GridSize>,
    max_step_size: u8,
}

impl<R> Default for GridWalker2DBuilder<R>
where
    R: Rng,
{
    fn default() -> Self {
        Self {
            current_pos: None,
            rng: None,
            size: None,
            max_step_size: 1,
        }
    }
}

impl<R> GridWalker2DBuilder<R>
where
    R: Rng,
{
    /// Set up starting position for the walker algorithm.
    pub fn with_current_pos(mut self, current_pos: GridPos2D) -> Self {
        self.current_pos = Some(current_pos);
        self
    }

    /// Provide the [Rng] for random generation.
    pub fn with_rng(mut self, rng: R) -> Self {
        self.rng = Some(rng);
        self
    }

    /// Set up maximum step size: at every iteration the Walker will pick a [GridDir] and make `1..n` steps in that direction at random.
    pub fn with_max_step_size(mut self, max_step_size: u8) -> Self {
        self.max_step_size = max_step_size;
        self
    }

    /// Set up [GridSize] for walker to walk inside.
    pub fn with_size(mut self, size: GridSize) -> Self {
        self.size = Some(size);
        self
    }

    pub fn build(self) -> Result<GridWalker2D<R>, BuilderError> {
        let mut error = BuilderError::default();

        if self.size.is_none() {
            error.add_missing_field("size");
        }

        if self.current_pos.is_none() {
            error.add_missing_field("current_pos");
        }

        if self.rng.is_none() {
            error.add_missing_field("rng");
        }

        error.try_throw()?;

        let mut walked = HashSet::new();
        walked.insert(self.current_pos.unwrap());

        Ok(GridWalker2D {
            current_pos: self.current_pos.unwrap(),
            walked,
            rng: self.rng.unwrap(),
            size: self.size.unwrap(),
            max_step_size: self.max_step_size,
            iters: 0,
        })
    }
}

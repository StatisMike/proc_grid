use rand::{distributions::{Distribution, Uniform}, Rng};
use std::collections::HashSet;

use crate::{
    error::BuilderError,
    map::{GridSize, GridDir, GridMap2D},
    tile::GridTile2D,
    GridPos2D,
};

/// Struct implementing the random walker algorithm, producing the collection of [GridPos2D]. To be created with
/// [GridWalker2DBuilder].
pub struct GridWalker2D<R>
where
    R: Rng,
{
    current_pos: GridPos2D,
    walked: HashSet<GridPos2D>,
    rng: R,
    dir_rng: Uniform<usize>,
    step_rng: Option<Uniform<usize>>,
    size: GridSize,
    step_size: usize,
    iters: u32,
}

impl<R> GridWalker2D<R>
where
    R: Rng,
{
    /// Number of calls to the [Self::walk()] method.
    pub fn current_iters(&self) -> u32 {
        self.iters
    }

    ///
    pub fn walk(&mut self) -> bool {
        self.iters += 1;
        let idx: usize = self.dir_rng.sample(&mut self.rng);

        let step_size = if let Some(step_size_rng) = self.step_rng {
            step_size_rng.sample(&mut self.rng)
        } else {
            self.step_size
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

    /// Generate [GridMap2D] out of gathered [GridPos2D].
    ///
    /// # Arguments
    ///
    /// - `tile_fun` - function which will generate the [GridTile2D]-implementing objects with specified positions.
    pub fn gen_grid_map<T>(&self, tile_fn: fn(GridPos2D) -> T) -> GridMap2D<T>
    where
        T: GridTile2D,
    {
        let mut map = GridMap2D::new(self.size);

        for pos in self.walked.iter() {
            map.insert_tile(tile_fn(*pos));
        }
        map
    }

    pub fn set_current_pos(&mut self, current_pos: GridPos2D) {
        self.current_pos = current_pos;
    }

    pub fn current_pos(&self) -> GridPos2D {
        self.current_pos
    }

    pub fn reset(&mut self) {
        self.iters = 0;
        self.walked.clear();
    }
}

pub struct GridWalker2DBuilder<R>
where
    R: Rng,
{
    current_pos: Option<GridPos2D>,
    rng: Option<R>,
    size: Option<GridSize>,
    min_step_size: usize,
    max_step_size: usize,
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
            min_step_size: 1,
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

    /// Set up minimum step size: at every iteration the Walker will pick a [GridDir] and make `min..max` steps in that direction at random.
    pub fn with_min_step_size(mut self, min_step_size: usize) -> Self {
        self.min_step_size = min_step_size;
        self
    }

    /// Set up maximum step size: at every iteration the Walker will pick a [GridDir] and make `min..max` steps in that direction at random.
    pub fn with_max_step_size(mut self, max_step_size: usize) -> Self {
        self.max_step_size = max_step_size;
        self
    }

    /// Set up [GridSize] for walker to walk inside.
    pub fn with_size(mut self, size: GridSize) -> Self {
        self.size = Some(size);
        self
    }

    pub fn build(self) -> Result<GridWalker2D<R>, BuilderError> {
        let mut error = BuilderError::new();

        if self.size.is_none() {
            error.add_missing_field("size");
        }

        let current_pos = if let Some(pos) = self.current_pos {
            pos
        } else {
            self.size.unwrap().center()
        };

        if self.rng.is_none() {
            error.add_missing_field("rng");
        }

        error.try_throw()?;

        let dir_rng = rand::distributions::Uniform::new(0, GridDir::ALL.len());
        let step_rng = self.get_step_rng();

        let mut walked = HashSet::new();
        walked.insert(current_pos);

        Ok(GridWalker2D {
            current_pos,
            walked,
            rng: self.rng.unwrap(),
            size: self.size.unwrap(),
            dir_rng,
            step_rng,
            step_size: self.min_step_size,
            iters: 0,
        })
    }

    fn get_step_rng(&self) -> Option<Uniform<usize>> {
        if self.min_step_size == self.max_step_size {
            return None;
        }

        Some(rand::distributions::Uniform::new(self.min_step_size, self.max_step_size + 1))
    }
}

use crate::GridPos2D;

#[derive(Debug, Clone, Copy)]
pub struct GridSize {
    x: u32,
    y: u32,
    center: GridPos2D,
}

impl GridSize {
    pub fn new(x: u32, y: u32) -> Self {
        let center = Self::calc_center_approx(x, y);
        Self { x, y, center }
    }

    pub fn x(&self) -> u32 {
        self.x
    }

    pub fn y(&self) -> u32 {
        self.y
    }

    pub fn center(&self) -> GridPos2D {
        self.center
    }

    pub fn is_position_valid(&self, position: &GridPos2D) -> bool {
        position.0 < self.x && position.1 < self.y
    }

    pub fn get_all_possible_positions(&self) -> Vec<GridPos2D> {
        let mut out = Vec::new();

        for x in 0..self.x {
            for y in 0..self.y {
                out.push((x, y));
            }
        }

        out
    }

    /// Get Position distance from border
    pub fn distance_from_border(&self, position: &GridPos2D) -> u32 {
        *[
            position.0,
            self.x - position.0 - 1,
            position.1,
            self.y - position.1 - 1,
        ]
        .iter()
        .min()
        .unwrap()
    }

    /// Get Position distance from center.
    pub fn distance_from_center(&self, position: &GridPos2D) -> u32 {
        if self.center.0 < position.0 {
            position.0 - self.center.0
        } else {
            self.center.0 - position.0
        }
        .min(if self.center.1 < position.1 {
            position.1 - self.center.1
        } else {
            self.center.1 - position.1
        })
    }

    fn calc_center_approx(x: u32, y: u32) -> GridPos2D {
        (x / 2, y / 2)
    }
}

use std::ops::{Add, AddAssign, Sub};

pub mod identifiable;

#[cfg(feature = "vis")]
pub mod vis;

#[derive(Debug)]
pub struct GridTile<Data>
where
    Data: TileData,
{
    position: GridPosition,
    data: Data,
}

/// Marker trait for structs that can be contained withing [`GridMap2D`](crate::map::GridMap2D) and [`TileContainer`]
pub trait TileData: Sized {}

/// Container of the [`TileData`] owning the data, when it is not yet passed to [`GridMap2D`](crate::map::GridMap2D) or
/// after it was retrieved from it through [`drain`](crate::map::GridMap2D::drain) or [`drain_remapped`](crate::map::GridMap2D::drain_remapped)
impl<Data: TileData> GridTile<Data> {
    #[inline]
    pub fn new(position: GridPosition, data: Data) -> Self {
        Self { position, data }
    }

    pub fn into_inner(self) -> Data {
        self.data
    }
}

impl<Data: TileData> TileContainer for GridTile<Data> {
    fn grid_position(&self) -> GridPosition {
        self.position
    }
}

impl<Data: TileData> AsRef<Data> for GridTile<Data> {
    fn as_ref(&self) -> &Data {
        &self.data
    }
}

impl<Data: TileData> AsMut<Data> for GridTile<Data> {
    fn as_mut(&mut self) -> &mut Data {
        &mut self.data
    }
}

/// Reference to the [`TileData`] contained within [`GridMap2D`](crate::map::GridMap2D). Underlying tile data
/// can be accessed through [`AsRef`] implementation.
pub struct GridTileRef<'a, Data>
where
    Data: TileData,
{
    position: GridPosition,
    data: &'a Data,
}

impl<'a, Data: TileData> AsRef<Data> for GridTileRef<'a, Data> {
    fn as_ref(&self) -> &'a Data {
        self.data
    }
}

impl<'a, Data: TileData> GridTileRef<'a, Data> {
    #[inline]
    pub fn new(position: GridPosition, data: &'a Data) -> Self {
        Self { position, data }
    }

    pub fn inner(&self) -> &Data {
        self.data
    }

    pub(crate) fn maybe_new(position: GridPosition, maybe_data: Option<&'a Data>) -> Option<Self> {
        maybe_data.map(|data| Self { position, data })
    }
}

impl<Data: TileData> TileContainer for GridTileRef<'_, Data> {
    fn grid_position(&self) -> GridPosition {
        self.position
    }
}

/// Mutable reference to the [`TileData`] contained within [`GridMap2D`](crate::map::GridMap2D). Underlying tile data
/// can be accessed through [`AsRef`] and [`AsMut`] implementations.
pub struct GridTileRefMut<'a, Data>
where
    Data: TileData,
{
    position: GridPosition,
    data: &'a mut Data,
}

impl<'a, Data: TileData> AsRef<Data> for GridTileRefMut<'a, Data> {
    fn as_ref(&self) -> &Data {
        self.data
    }
}

impl<'a, Data: TileData> AsMut<Data> for GridTileRefMut<'a, Data> {
    fn as_mut(&mut self) -> &mut Data {
        self.data
    }
}

impl<'a, Data: TileData> GridTileRefMut<'a, Data> {
    #[inline]
    pub fn new(position: GridPosition, data: &'a mut Data) -> Self {
        Self { position, data }
    }

    pub fn inner(&self) -> &Data {
        self.data
    }

    pub fn inner_mut(&mut self) -> &mut Data {
        self.data
    }

    pub(crate) fn maybe_new(
        position: GridPosition,
        maybe_data: Option<&'a mut Data>,
    ) -> Option<Self> {
        maybe_data.map(|data| Self { position, data })
    }
}

impl<Data: TileData> TileContainer for GridTileRefMut<'_, Data> {
    fn grid_position(&self) -> GridPosition {
        self.position
    }
}

/// Position of the [`TileData`] within a [`GridMap2D`](crate::map::GridMap2D).
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct GridPosition {
    x: u32,
    y: u32,
    z: Option<u32>,
}

impl GridPosition {
    #[inline]
    pub fn new_xy(x: u32, y: u32) -> Self {
        Self { x, y, z: None }
    }

    #[inline]
    pub fn new_xyz(x: u32, y: u32, z: u32) -> Self {
        Self { x, y, z: Some(z) }
    }

    #[inline]
    pub fn x(&self) -> &u32 {
        &self.x
    }

    #[inline]
    pub fn y(&self) -> &u32 {
        &self.y
    }

    #[inline]
    pub fn xy(&self) -> (u32, u32) {
        (self.x, self.y)
    }

    #[inline]
    pub fn z(&self) -> &Option<u32> {
        &self.z
    }

    pub fn add_xy(&mut self, xy: (u32, u32)) {
        self.x += xy.0;
        self.y += xy.1;
    }

    pub fn add_z(&mut self, z: u32) {
        if let Some(zs) = &mut self.z {
            *zs += z;
        }
    }

    pub fn in_range(&self, other: &Self, range: u32) -> bool {
        let mut distance = 0;

        if let (Some(zl), Some(zr)) = (self.z(), other.z()) {
            distance += zl.max(zr) - zl.min(zr);
        }

        if distance > range {
            return false;
        }

        distance += self.x().max(other.x()) - self.x().min(other.x());

        if distance > range {
            return false;
        }

        (distance + self.y().max(other.y()) - self.y().min(other.y())) <= range
    }

    pub fn generate_rect_area(upper_left: &Self, lower_right: &Self) -> Vec<Self> {
        let mut out = Vec::new();

        let layers_range = match (*upper_left.z(), *lower_right.z()) {
            (None, None) => None,
            (None, Some(z)) => Some(z..=z),
            (Some(z), None) => Some(z..=z),
            (Some(z1), Some(z2)) => Some(z1..=z2),
        };

        if let Some(z_range) = layers_range.clone() {
            for z in z_range {
                for x in *upper_left.x()..=*lower_right.x() {
                    for y in *upper_left.y()..=*lower_right.y() {
                        out.push(GridPosition::new_xyz(x, y, z));
                    }
                }
            }
        } else {
            for x in *upper_left.x()..=*lower_right.x() {
                for y in *upper_left.y()..=*lower_right.y() {
                    out.push(GridPosition::new_xy(x, y));
                }
            }
        }
        out
    }
}

impl Add for GridPosition {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        if let (Some(lz), Some(rz)) = (&self.z, &rhs.z) {
            Self::new_xyz(self.x + rhs.x, self.y + rhs.y, lz + rz)
        } else {
            Self::new_xy(self.x + rhs.x, self.y + rhs.y)
        }
    }
}

impl Sub for GridPosition {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (&self.z, &rhs.z) {
            (None, None) => Self::new_xy(self.x - rhs.x, self.y - rhs.y),
            (None, Some(z)) => Self::new_xyz(self.x - rhs.x, self.y - rhs.y, *z),
            (Some(z), None) => Self::new_xyz(self.x - rhs.x, self.y - rhs.y, *z),
            (Some(lz), Some(rz)) => Self::new_xyz(self.x - rhs.x, self.y - rhs.y, lz - rz),
        }
    }
}

impl AddAssign for GridPosition {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;

        if let (Some(mut lz), Some(rz)) = (&mut self.z, &rhs.z) {
            lz += rz;
        }
    }
}

/// Trait gathering the containers for [`TileData`] outside of the [`GridMap2D`](crate::map::GridMap2D).
///
/// Allows accessing all the data not contained within tile data, making sense only in context of the grid map.
pub trait TileContainer {
    fn grid_position(&self) -> GridPosition;
}

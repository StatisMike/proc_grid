use std::ops::{Add, AddAssign};

pub mod identifiable;

#[cfg(feature = "vis")]
pub mod vis;

#[derive(Debug)]
pub struct GridTile<Data>
where Data: TileData
{
    position: GridPosition,
    data: Data
}

pub trait TileData: Sized {}

impl<Data: TileData> GridTile<Data>
{
    pub fn new(position: GridPosition, data: Data) -> Self
    {
        Self { position, data }
    }

    pub fn inner(&self) -> &Data
    {
        &self.data
    }

    pub fn inner_mut(&mut self) -> &mut Data
    {
        &mut self.data
    }

    pub fn into_inner(self) -> Data
    {
        self.data
    }
}

impl <Data: TileData>WithTilePosition for GridTile<Data>
{
    fn grid_position(&self) -> GridPosition {
        self.position
    }
}

pub struct GridTileRef<'a, Data>
where Data: TileData
{
    position: GridPosition,
    data: &'a Data
}

impl<'a, Data: TileData> GridTileRef<'a, Data>
{
    pub fn new(position: GridPosition, data: &'a Data) -> Self
    {
        Self { position, data }
    }

    pub fn inner(&self) -> &Data
    {
        &self.data
    }

    pub (crate) fn maybe_new(position: GridPosition, maybe_data: Option<&'a Data>) -> Option<Self>
    {
        maybe_data.map(|data| { Self { position, data }})
    }
}

impl <Data: TileData>WithTilePosition for GridTileRef<'_, Data>
{
    fn grid_position(&self) -> GridPosition {
        self.position
    }
}

pub struct GridTileRefMut<'a, Data>
where Data: TileData
{
    position: GridPosition,
    data: &'a mut Data
}

impl<'a, Data: TileData> GridTileRefMut<'a, Data>
{
    pub fn new(position: GridPosition, data: &'a mut Data) -> Self
    {
        Self { position, data }
    }

    pub fn inner(&self) -> &Data
    {
        &self.data
    }

    pub fn inner_mut(&mut self) -> &mut Data
    {
        &mut self.data
    }

    pub (crate) fn maybe_new(position: GridPosition, maybe_data: Option<&'a mut Data>) -> Option<Self>
    {
        maybe_data.map(|data| { Self { position, data }})
    }
}

impl <Data: TileData>WithTilePosition for GridTileRefMut<'_, Data>
{
    fn grid_position(&self) -> GridPosition {
        self.position
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct GridPosition {
    x: u32,
    y: u32,
    z: Option<u32>
}

impl GridPosition {
    pub fn new_xy(x: u32, y: u32) -> Self {
        Self { x, y, z: None }
    }

    pub fn new_xyz(x: u32, y: u32, z: u32) -> Self {
        Self { x, y, z: Some(z) }
    }

    pub fn x(&self) -> &u32 {
        &self.x
    }
    pub fn y(&self) -> &u32 {
        &self.y
    }
    pub fn xy(&self) -> (u32, u32) {
        (self.x, self.y)
    }
    pub fn z(&self) -> &Option<u32> {
        &self.z
    }

    pub fn add_xy(&mut self, xy: (u32, u32))
    {
        self.x += xy.0;
        self.y += xy.1;
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

impl AddAssign for GridPosition {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;

        if let (Some(mut lz), Some(rz)) = (&mut self.z, &rhs.z) {
            lz += rz;
        }
    }
}

pub trait WithTilePosition {
    fn grid_position(&self) -> GridPosition;
}
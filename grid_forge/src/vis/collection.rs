use std::{
    collections::{hash_map::Entry, HashMap},
    error::Error,
    fmt::Display,
    marker::PhantomData,
};

use image::{ImageBuffer, Pixel};

use crate::{
    map::{GridMap2D, GridSize},
    tile::{
        identifiable::IdentifiableTile, vis::{DefaultPixel, VisTile2D}
    },
    GridPos2D,
};

use super::{read_tile, write_tile, EmptyTile};

pub enum VisCollectionOutcome<P, const WIDTH: usize, const HEIGHT: usize>
where P: DefaultPixel
{
    Empty,
    Added,
    Existing,
    Replaced([[P; WIDTH]; HEIGHT])
}

pub type VisCollectionResult<P, const WIDTH: usize, const HEIGHT: usize> = Result<VisCollectionOutcome<P, WIDTH, HEIGHT>, VisError>;

#[derive(Debug, Clone)]
pub struct VisCollection<T, P, const WIDTH: usize, const HEIGHT: usize>
where
    T: IdentifiableTile,
    P: Pixel + DefaultPixel,
{
    inner: HashMap<u64, [[P; WIDTH]; HEIGHT]>,
    empty: Option<EmptyTile<P, WIDTH, HEIGHT>>,
    tile: PhantomData<T>,
}

impl<T, P, const WIDTH: usize, const HEIGHT: usize> Default for VisCollection<T, P, WIDTH, HEIGHT>
where
    T: IdentifiableTile,
    P: Pixel + DefaultPixel,
{
    fn default() -> Self {
        Self {
            inner: Default::default(),
            empty: None,
            tile: PhantomData::<T>,
        }
    }
}

impl<T, P, const WIDTH: usize, const HEIGHT: usize> VisCollection<T, P, WIDTH, HEIGHT>
where
    T: IdentifiableTile,
    P: Pixel + DefaultPixel,
{
    // ----- Input ----- //

    /// Sets pixels which should be ignored while reading the tiles.
    ///
    /// Useful while reading image containing some tiles which should be treated as empty. If such tile is encountered,
    /// it is omitted during reading.
    pub fn set_empty_tile_pixels(&mut self, pixels: Option<[[P; WIDTH]; HEIGHT]>) {
        if let Some(pix) = pixels {
            self.empty = Some(EmptyTile::new(pix));
        } else {
            self.empty = None
        }
    }

    /// Add pixels for [`IdentifiableTile`] implementing [`VisTile2D`] if none were collected yet.
    ///
    /// # Returns
    /// - `true` if the tiles were not present and were added
    ///
    /// # See also
    /// - [`Self::set_vis_tile_pixels`]
    /// - [`Self::add_tiles_from_vis_map`]
    pub fn add_vis_tile_pixels<V: VisTile2D<P, WIDTH, HEIGHT> + IdentifiableTile>(
        &mut self,
        tile: &V,
    ) -> VisCollectionOutcome<P, WIDTH, HEIGHT> {
        let inner = &mut self.inner;
        if let Entry::Vacant(e) = inner.entry(tile.get_tile_id()) {
            let pix = tile.vis_pixels();
            if Self::check_empty_id(&self.empty, tile.get_tile_id())
                || Self::check_empty_pix(&self.empty, &pix)
            {
                return VisCollectionOutcome::Empty;
            }
            e.insert(pix);
            return VisCollectionOutcome::Added;
        }
        VisCollectionOutcome::Existing
    }

    /// Set pixels for [`IdentifiableTile`] implementing [`VisTile2D`] unconditionally.
    ///
    /// # Returns
    /// - pixels which were replaced, if some were present already. `None` otherwise.
    ///
    /// # See also
    /// - [`Self::add_vis_tile_pixels`]
    /// - [`Self::add_tiles_from_vis_map`]
    pub fn set_vis_tile_pixels<V: VisTile2D<P, WIDTH, HEIGHT> + IdentifiableTile>(
        &mut self,
        tile: &V,
    ) -> VisCollectionOutcome<P, WIDTH, HEIGHT> {
        let tile_id = tile.get_tile_id();
        let pix = tile.vis_pixels();
        if !Self::check_empty_id(&self.empty, tile_id) && !Self::check_empty_pix(&self.empty, &pix)
        {
            match self.inner.insert(tile_id, pix) {
                Some(pixels) => VisCollectionOutcome::Replaced(pixels),
                None => VisCollectionOutcome::Added,
            }
        } else {
            VisCollectionOutcome::Empty
        }
    }

    /// Add pixels for tiles from [GridMap2D], if the tiles contained were implementing [IdentifiableTile] and [VisTile2D] unconditionally.
    ///
    /// # See also
    /// - [`Self::add_vis_tile_pixels`]
    /// - [`Self::set_vis_tile_pixels`]
    pub fn add_tiles_from_vis_map<V: VisTile2D<P, WIDTH, HEIGHT> + IdentifiableTile>(
        &mut self,
        grid_map: &GridMap2D<V>,
    ) {
        for position in grid_map.get_all_positions() {
            if let Some(tile) = grid_map.get_tile_at_position(&position) {
                self.add_vis_tile_pixels(tile);
            }
        }
    }

    pub fn add_tile_pixels(
        &mut self,
        tile: &T,
        buffer: &ImageBuffer<P, Vec<P::Subpixel>>,
    ) -> VisCollectionResult<P, WIDTH, HEIGHT> {
        if let Entry::Vacant(e) = self.inner.entry(tile.get_tile_id()) {
            if Self::check_empty_id(&self.empty, tile.get_tile_id()) {
                return Ok(VisCollectionOutcome::Empty);
            }
            let pixels = Self::read_pixels_for_tile_at_pos(buffer, tile.grid_position())?;
            e.insert(pixels);
            return Ok(VisCollectionOutcome::Added);
        }
        Ok(VisCollectionOutcome::Existing)
    }

    pub fn set_tile_pixels(
        &mut self,
        tile: &T,
        buffer: &ImageBuffer<P, Vec<P::Subpixel>>,
    ) -> VisCollectionResult<P, WIDTH, HEIGHT>  {
        if Self::check_empty_id(&self.empty, tile.get_tile_id()) {
            return Ok(VisCollectionOutcome::Empty);
        }
        let pixels = Self::read_pixels_for_tile_at_pos(buffer, tile.grid_position())?;
        Ok(self.set_tile_pixels_manual(tile.get_tile_id(), pixels))
    }

    pub fn add_tiles_from_map(
        &mut self,
        grid_map: &GridMap2D<T>,
        buffer: &ImageBuffer<P, Vec<P::Subpixel>>,
    ) -> Result<(), VisError> {
        for position in grid_map.get_all_positions() {
            if let Some(tile) = grid_map.get_tile_at_position(&position) {
                self.add_tile_pixels(tile, buffer)?;
            }
        }
        Ok(())
    }

    pub fn add_tile_pixels_manual(&mut self, tile_id: u64, pixels: [[P; WIDTH]; HEIGHT]) -> VisCollectionOutcome<P, WIDTH, HEIGHT> {
        if let Entry::Vacant(e) = self.inner.entry(tile_id) {
            if Self::check_empty_pix(&self.empty, &pixels) {
                return VisCollectionOutcome::Empty;
            }
            e.insert(pixels);
            return VisCollectionOutcome::Added;
        }
        VisCollectionOutcome::Existing
    }

    pub fn set_tile_pixels_manual(
        &mut self,
        tile_id: u64,
        pixels: [[P; WIDTH]; HEIGHT],
    ) -> VisCollectionOutcome<P, WIDTH, HEIGHT>  {
        if Self::check_empty_pix(&self.empty, &pixels) {
            return VisCollectionOutcome::Empty;
        }
        match self.inner.insert(tile_id, pixels) {
            Some(pixels) => VisCollectionOutcome::Replaced(pixels),
            None => VisCollectionOutcome::Added,
        }
    }

    //----- Output -----//

    pub fn draw_tile(
        &self,
        tile: &T,
        buffer: &mut ImageBuffer<P, Vec<P::Subpixel>>,
    ) -> Result<(), VisError> {
        if let Some(pixels) = self.inner.get(&tile.get_tile_id()) {
            if let Err(err) = write_tile(buffer, tile.grid_position(), pixels) {
                return Err(VisError::Write {
                    tile_pos: err.tile_pos,
                    pixel_pos: err.pixel_pos,
                });
            }
            return Ok(());
        }
        Err(VisError::NoPixelsError(tile.get_tile_id()))
    }

    pub fn init_map_image_buffer(&self, grid_size: &GridSize) -> ImageBuffer<P, Vec<P::Subpixel>> {

        ImageBuffer::new(grid_size.x() * WIDTH as u32, grid_size.y() * HEIGHT as u32)
    }

    pub fn draw_map(
        &self,
        grid_map: &GridMap2D<T>,
        buffer: &mut ImageBuffer<P, Vec<P::Subpixel>>,
    ) -> Result<(), VisError> {
        for position in grid_map.get_all_positions() {
            if let Some(tile) = grid_map.get_tile_at_position(&position) {
                self.draw_tile(tile, buffer)?;
            }
        }
        Ok(())
    }

    // ------ Private ------ //
    fn read_pixels_for_tile_at_pos(
        buffer: &ImageBuffer<P, Vec<P::Subpixel>>,
        pos: GridPos2D,
    ) -> Result<[[P; WIDTH]; HEIGHT], VisError> {
        let mut pixels = [[P::pix_default(); WIDTH]; HEIGHT];
        if let Err(err) = read_tile(&mut pixels, buffer, pos) {
            return Err(VisError::Read {
                tile_pos: err.tile_pos,
                pixel_pos: err.pixel_pos,
            });
        }
        Ok(pixels)
    }

    fn check_empty_id(empty_tile: &Option<EmptyTile<P, WIDTH, HEIGHT>>, tile_id: u64) -> bool {
        if let Some(empty) = empty_tile {
            return empty.tile_id == tile_id;
        }
        false
    }

    fn check_empty_pix(
        empty_tile: &Option<EmptyTile<P, WIDTH, HEIGHT>>,
        pixels: &[[P; WIDTH]; HEIGHT],
    ) -> bool {
        if let Some(empty) = empty_tile {
            return &empty.pixels == pixels;
        }
        false
    }
}

// ---------- Errors ---------- //
#[derive(Debug)]
pub enum VisError {
    NoPixelsError(u64),
    Read {
        tile_pos: GridPos2D,
        pixel_pos: (u32, u32),
    },
    Write {
        tile_pos: GridPos2D,
        pixel_pos: (u32, u32),
    },
}

impl Display for VisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoPixelsError(tile_id) => write!(
                    f,
                    "cannot draw tile: no pixels for tile of id: {tile_id} is present"
                ),
            Self::Read {
                tile_pos,
                pixel_pos,
            } => write!(f, "cannot read tile pixels: image buffer is out of bounds for tile on position: {tile_pos:?}, with pixel: {pixel_pos:?}"),
            Self::Write {
                tile_pos,
                pixel_pos,
            } => write!(f, "cannot draw tile: image buffer is out of bounds for tile on position: {tile_pos:?}, with pixel: {pixel_pos:?}"),
        }
    }
}

impl Error for VisError {}

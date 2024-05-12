use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::marker::PhantomData;

use image::{ImageBuffer, Pixel};

use crate::map::{GridMap2D, GridSize};
use crate::tile::identifiable::IdentifiableTile;
use crate::tile::vis::{PixelWithDefault, VisTile2D};
use crate::GridPos2D;

use super::error::VisError;
use super::ops::create_tile_id_from_pixels;
use super::{read_tile, write_tile, EmptyTile};

/// Outcome of `set_*` and `add_*` methods of [`VisCollection`].
pub enum VisCollectionOutcome<P, const WIDTH: usize, const HEIGHT: usize>
where
    P: PixelWithDefault,
{
    /// Passed pixels were identified as registered for empty tile.
    Empty,
    /// Passed pixels were set successfully.
    Added,
    /// Passed pixels were not set, as some were already registered for given `tile_id`.
    Existing,
    /// Passed pixels were set and overwrote some already registered for given `tile_id`. Contains overwritten pixels.
    Replaced([[P; WIDTH]; HEIGHT]),
}

pub type VisCollectionResult<P, const WIDTH: usize, const HEIGHT: usize> =
    Result<VisCollectionOutcome<P, WIDTH, HEIGHT>, VisError<WIDTH, HEIGHT>>;

/// Collection of pixels registered for identifiers of [`IdentifiableTile`] struct.
#[derive(Debug, Clone)]
pub struct VisCollection<T, P, const WIDTH: usize, const HEIGHT: usize>
where
    T: IdentifiableTile,
    P: Pixel + PixelWithDefault,
{
    /// Contains provided or created `type_id` alongside the pixels of given tile.
    inner: HashMap<u64, [[P; WIDTH]; HEIGHT]>,
    /// Lookup table for created `type_id` alongside the value for the key. For safety in situations
    /// where there was provided `type_id` differing for the one that is created from pixels.
    rev: HashMap<u64, u64>,
    /// Optional specification for pixels which should be omitted during load.
    empty: Option<EmptyTile<P, WIDTH, HEIGHT>>,
    /// Marker for analyzed tile struct type.
    tile: PhantomData<T>,
}

impl<T, P, const WIDTH: usize, const HEIGHT: usize> Default for VisCollection<T, P, WIDTH, HEIGHT>
where
    T: IdentifiableTile,
    P: Pixel + PixelWithDefault,
{
    fn default() -> Self {
        Self {
            inner: Default::default(),
            rev: Default::default(),
            empty: None,
            tile: PhantomData::<T>,
        }
    }
}

impl<T, P, const WIDTH: usize, const HEIGHT: usize> VisCollection<T, P, WIDTH, HEIGHT>
where
    T: IdentifiableTile,
    P: Pixel + PixelWithDefault,
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
        let rev = &mut self.rev;
        if let Entry::Vacant(e) = inner.entry(tile.tile_type_id()) {
            let pix = tile.vis_pixels();
            if Self::check_empty_id(&self.empty, tile.tile_type_id())
                || Self::check_empty_pix(&self.empty, &pix)
            {
                return VisCollectionOutcome::Empty;
            }
            e.insert(pix);
            let to_remove = rev.insert(create_tile_id_from_pixels(&pix), tile.tile_type_id());
            if let Some(derived_id) = to_remove {
                rev.remove(&derived_id);
            }
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
        let tile_id = tile.tile_type_id();
        let pix = tile.vis_pixels();
        if !Self::check_empty_id(&self.empty, tile_id) && !Self::check_empty_pix(&self.empty, &pix)
        {
            self.rev
                .insert(create_tile_id_from_pixels(&pix), tile.tile_type_id());
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
        if let Entry::Vacant(e) = self.inner.entry(tile.tile_type_id()) {
            if Self::check_empty_id(&self.empty, tile.tile_type_id()) {
                return Ok(VisCollectionOutcome::Empty);
            }
            let pixels = Self::read_pixels_for_tile_at_pos(buffer, &tile.grid_position())?;
            e.insert(pixels);
            self.rev
                .insert(create_tile_id_from_pixels(&pixels), tile.tile_type_id());
            return Ok(VisCollectionOutcome::Added);
        }
        Ok(VisCollectionOutcome::Existing)
    }

    pub fn set_tile_pixels(
        &mut self,
        tile: &T,
        buffer: &ImageBuffer<P, Vec<P::Subpixel>>,
    ) -> VisCollectionResult<P, WIDTH, HEIGHT> {
        if Self::check_empty_id(&self.empty, tile.tile_type_id()) {
            return Ok(VisCollectionOutcome::Empty);
        }
        let pixels = Self::read_pixels_for_tile_at_pos(buffer, &tile.grid_position())?;
        self.rev
            .insert(create_tile_id_from_pixels(&pixels), tile.tile_type_id());
        Ok(self.set_tile_pixels_manual(tile.tile_type_id(), pixels))
    }

    pub fn add_tiles_from_map(
        &mut self,
        grid_map: &GridMap2D<T>,
        buffer: &ImageBuffer<P, Vec<P::Subpixel>>,
    ) -> Result<(), VisError<WIDTH, HEIGHT>> {
        for position in grid_map.get_all_positions() {
            if let Some(tile) = grid_map.get_tile_at_position(&position) {
                self.add_tile_pixels(tile, buffer)?;
            }
        }
        Ok(())
    }

    pub fn add_tile_pixels_manual(
        &mut self,
        tile_id: u64,
        pixels: [[P; WIDTH]; HEIGHT],
    ) -> VisCollectionOutcome<P, WIDTH, HEIGHT> {
        if let Entry::Vacant(e) = self.inner.entry(tile_id) {
            if Self::check_empty_pix(&self.empty, &pixels) {
                return VisCollectionOutcome::Empty;
            }
            e.insert(pixels);
            self.rev
                .insert(create_tile_id_from_pixels(&pixels), tile_id);
            return VisCollectionOutcome::Added;
        }
        VisCollectionOutcome::Existing
    }

    pub fn set_tile_pixels_manual(
        &mut self,
        tile_id: u64,
        pixels: [[P; WIDTH]; HEIGHT],
    ) -> VisCollectionOutcome<P, WIDTH, HEIGHT> {
        if Self::check_empty_pix(&self.empty, &pixels) {
            return VisCollectionOutcome::Empty;
        }
        self.rev
            .insert(create_tile_id_from_pixels(&pixels), tile_id);
        match self.inner.insert(tile_id, pixels) {
            Some(pixels) => VisCollectionOutcome::Replaced(pixels),
            None => VisCollectionOutcome::Added,
        }
    }

    //----- Output -----//
    pub fn get_tile_id_by_pixels(&self, pixels: &[[P; WIDTH]; HEIGHT]) -> Option<&u64> {
        self.rev.get(&create_tile_id_from_pixels(pixels))
    }

    pub fn is_empty(&self, check_pixels: &[[P; WIDTH]; HEIGHT]) -> bool {
        if let Some(EmptyTile { tile_id: _, pixels }) = self.empty {
            &pixels == check_pixels
        } else {
            false
        }
    }

    pub fn draw_tile(
        &self,
        tile: &T,
        buffer: &mut ImageBuffer<P, Vec<P::Subpixel>>,
    ) -> Result<(), VisError<WIDTH, HEIGHT>> {
        if let Some(pixels) = self.inner.get(&tile.tile_type_id()) {
            write_tile(buffer, tile.grid_position(), pixels)?;
            return Ok(());
        }
        Err(VisError::new_nopix(tile.tile_type_id()))
    }

    pub fn init_map_image_buffer(&self, grid_size: &GridSize) -> ImageBuffer<P, Vec<P::Subpixel>> {
        ImageBuffer::new(grid_size.x() * WIDTH as u32, grid_size.y() * HEIGHT as u32)
    }

    pub fn draw_map(
        &self,
        grid_map: &GridMap2D<T>,
        buffer: &mut ImageBuffer<P, Vec<P::Subpixel>>,
    ) -> Result<(), VisError<WIDTH, HEIGHT>> {
        for position in grid_map.get_all_positions() {
            if let Some(tile) = grid_map.get_tile_at_position(&position) {
                self.draw_tile(tile, buffer)?;
            }
        }
        Ok(())
    }

    // ------ Private ------ //
    pub(crate) fn read_pixels_for_tile_at_pos(
        buffer: &ImageBuffer<P, Vec<P::Subpixel>>,
        pos: &GridPos2D,
    ) -> Result<[[P; WIDTH]; HEIGHT], VisError<WIDTH, HEIGHT>> {
        let mut pixels = [[P::pix_default(); WIDTH]; HEIGHT];
        read_tile(&mut pixels, buffer, pos)?;
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

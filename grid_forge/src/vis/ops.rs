//! Various IO operations transforming between [`GridMap2D`] and [`ImageBuffer`] representation of grid map.

use std::hash::{DefaultHasher, Hash, Hasher};

use image::{ImageBuffer, Pixel};

use crate::map::{GridMap2D, GridSize};
use crate::tile::identifiable::builders::IdentTileBuilder;
use crate::tile::identifiable::IdentifiableTileData;

use super::collection::VisCollection;
use super::error::VisError;
use super::{PixelWithDefault, VisTile2D, VisTileData};

/// Easily load [`GridMap2D`] of [`IdentifiableTileData`]-implementing TileData, automatically saving each tile into provided
/// [`VisCollection`].
///
/// # Arguments
/// - `image_buffer` - an [`ImageBuffer`] containing the source image data.
/// - `collection` - a [`VisCollection`] containing the loaded tile pixels. New tile pixels will be automatically
/// inserted, calculating the `tile_type_id` on basis of tile pixels.
/// - `builder` - a struct which can be used to construct new tiles on basis of their `tile_id`. One of [`IdentTileBuilder`]
/// implementing objects.
///
/// # Warning
/// As the `tile_type_id` **is automatically calculated** with this function on basis of pixels, it won't work with specific,
/// manually declared identifiers. In this case, you need to use [`load_gridmap_identifiable_manual`].
pub fn load_gridmap_identifiable_auto<Data, P, B, const WIDTH: usize, const HEIGHT: usize>(
    image_buffer: &ImageBuffer<P, Vec<P::Subpixel>>,
    collection: &mut VisCollection<P, WIDTH, HEIGHT>,
    builder: &B,
) -> Result<GridMap2D<Data>, VisError<WIDTH, HEIGHT>>
where
    Data: IdentifiableTileData,
    P: PixelWithDefault + 'static,
    B: IdentTileBuilder<Data>,
{
    let size = check_grid_vis_size(image_buffer)?;
    let mut grid = GridMap2D::<Data>::new(size);

    for position in size.get_all_possible_positions() {
        let pixels = VisCollection::<P, WIDTH, HEIGHT>::read_pixels_for_tile_at_pos(
            image_buffer,
            &position,
        )?;
        let tile = builder.build_tile_unchecked(position, create_tile_id_from_pixels(&pixels));
        match collection.add_tile_pixels(&tile, image_buffer)? {
            super::collection::VisCollectionOutcome::Empty => {
                continue;
            }
            _ => grid.insert_tile(tile),
        };
    }
    Ok(grid)
}

/// Load [`GridMap2D`] of [`IdentifiableTileData`]-implementing struct using registered pixels in [`VisCollection`].
///
/// If you don't need to declare the `tile_type_id` yourself, you can use [`load_gridmap_identifiable_auto`], which generates
/// identifiers on basis of each pixel.
///
/// # Arguments
/// - `image_buffer` - an [`ImageBuffer`] containing the source image data.
/// - `collection` - a [`VisCollection`] containing the tile pixels. It needs to be populated before usage.
/// - `builder` - a struct which can be used to construct new tiles on basis of their `tile_id`. One of [`IdentTileBuilder`]
/// implementing objects.
pub fn load_gridmap_identifiable_manual<Data, P, B, const WIDTH: usize, const HEIGHT: usize>(
    image_buffer: &ImageBuffer<P, Vec<P::Subpixel>>,
    collection: &VisCollection<P, WIDTH, HEIGHT>,
    builder: &B,
) -> Result<GridMap2D<Data>, VisError<WIDTH, HEIGHT>>
where
    Data: IdentifiableTileData,
    P: PixelWithDefault + 'static,
    B: IdentTileBuilder<Data>,
{
    let size = check_grid_vis_size(image_buffer)?;
    let mut grid = GridMap2D::<Data>::new(size);

    for position in size.get_all_possible_positions() {
        let pixels = VisCollection::<P, WIDTH, HEIGHT>::read_pixels_for_tile_at_pos(
            image_buffer,
            &position,
        )?;
        if let Some(tile_id) = collection.get_tile_id_by_pixels(&pixels) {
            grid.insert_tile(builder.build_tile_unchecked(position, *tile_id));
        } else if !collection.is_empty(&pixels) {
            return Err(VisError::new_nonexist(position));
        }
    }
    Ok(grid)
}

/// Utility function to generate [`ImageBuffer`] of correct size for specific size of [`GridMap2D`] to write into
/// with [`write_gridmap_identifiable`] and [`write_gridmap_vis`].
pub fn init_map_image_buffer<P, const WIDTH: usize, const HEIGHT: usize>(
    grid_size: &GridSize,
) -> ImageBuffer<P, Vec<P::Subpixel>>
where
    P: PixelWithDefault,
{
    ImageBuffer::new(grid_size.x() * WIDTH as u32, grid_size.y() * HEIGHT as u32)
}

/// Write [`GridMap2D`] comprised of tiles containing [`IdentifiableTileData`] into provided [`ImageBuffer`], using
/// pixel data gatheed in [`VisCollection`].
///
/// To make sure that the image buffer has exact correct size needed to write the `GridMap2D` representation into it,
/// you can use [`check_grid_image_size`] beforehand.
pub fn write_gridmap_identifiable<Data, P, const WIDTH: usize, const HEIGHT: usize>(
    image_buffer: &mut ImageBuffer<P, Vec<P::Subpixel>>,
    grid_map: &GridMap2D<Data>,
    collection: &VisCollection<P, WIDTH, HEIGHT>,
) -> Result<(), VisError<WIDTH, HEIGHT>>
where
    Data: IdentifiableTileData,
    P: PixelWithDefault + 'static,
{
    collection.draw_map(grid_map, image_buffer)?;

    Ok(())
}

/// Write [`GridMap2D`] comprised of tiles containing [`VisTileData`] into provided [`ImageBuffer`]. Pixel data retrieved
/// via [`VisTileData::vis_pixels`] will be used.
///
/// To make sure that the image buffer has exact correct size needed to write the `GridMap2D` representation into it,
/// you can use [`check_grid_image_size`] beforehand.
pub fn write_gridmap_vis<Data, P, const WIDTH: usize, const HEIGHT: usize>(
    image_buffer: &mut ImageBuffer<P, Vec<P::Subpixel>>,
    grid_map: &GridMap2D<Data>,
) -> Result<(), VisError<WIDTH, HEIGHT>>
where
    Data: VisTileData<P, WIDTH, HEIGHT>,
    P: PixelWithDefault + 'static,
{
    for position in grid_map.get_all_positions() {
        grid_map
            .get_tile_at_position(&position)
            .expect("cannot get tile")
            .vis_to_buffer(image_buffer)?;
    }

    Ok(())
}

/// Checks the size of the [`ImageBuffer`] while loading [`GridMap2D`] from its visual representation, and produces
/// the [`GridSize`] inferred from the image size. Results in [`VisError`] if the image size is not compatible
/// with provided tile size in pixels.
pub fn check_grid_vis_size<P: Pixel + 'static, const WIDTH: usize, const HEIGHT: usize>(
    image: &ImageBuffer<P, Vec<P::Subpixel>>,
) -> Result<GridSize, VisError<WIDTH, HEIGHT>> {
    if image.height() as usize % HEIGHT != 0 || image.width() as usize % WIDTH != 0 {
        Err(VisError::new_grid_load(image.width(), image.height()))
    } else {
        Ok(GridSize::new_xy(
            image.width() / WIDTH as u32,
            image.height() / HEIGHT as u32,
        ))
    }
}

/// Checks the size of the [`ImageBuffer`] before writing [`GridMap2D`] visual representation into it. Results in
/// [`VisError`] if the image buffer size and [`GridSize`] does not match.
pub fn check_grid_image_size<P: Pixel + 'static, const WIDTH: usize, const HEIGHT: usize>(
    image: &ImageBuffer<P, Vec<P::Subpixel>>,
    size: &GridSize,
) -> Result<(), VisError<WIDTH, HEIGHT>> {
    let expected = (size.x() * WIDTH as u32, size.y() * HEIGHT as u32);

    if expected != (image.width(), image.height()) {
        Err(VisError::new_grid_save(
            expected,
            (image.width(), image.height()),
        ))
    } else {
        Ok(())
    }
}

// ------ PRIVATE ------ //

#[inline]
pub(crate) fn create_tile_id_from_pixels<
    P: PixelWithDefault,
    const WIDTH: usize,
    const HEIGHT: usize,
>(
    pixels: &[[P; WIDTH]; HEIGHT],
) -> u64 {
    let mut hasher = DefaultHasher::default();
    pixels.hash(&mut hasher);
    hasher.finish()
}

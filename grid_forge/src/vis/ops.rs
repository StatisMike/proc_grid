use std::hash::{DefaultHasher, Hash, Hasher};

use image::{ImageBuffer, Pixel};

use crate::{
    map::{GridMap2D, GridSize},
    tile::{
        identifiable::{builder::IdentTileBuilder, IdentifiableTile},
        vis::{DefaultPixel, VisTile2D},
    },
};

use super::{
    collection::VisCollection,
    error::VisError,
    write_tile,
};

/// Easily load [`GridMap2D`] of [`IdentifiableTile`] struct, automatically saving each tile into provided [`VisCollection`].
///
/// # Arguments
/// - `image_buffer` - an [ImageBuffer] containing the source image data.
/// - `collection` - a [`VisCollection`] containing the loaded tile pixels. New tile pixels will be automatically
/// inserted, calculating the `tile_id` on basis of tile pixels.
/// - `builder` - a struct which can be used to construct new tiles on basis of their `tile_id`. One of [`IdentTileBuilder`]
/// implementing objects.
///
/// # Warning
/// As the `tile_id` **is automatically calculated** with this function on basis of pixels, it won't work with specific,
/// manually declared identifiers. In this case, you need to use [`load_gridmap_identifiable_manual`].
pub fn load_gridmap_identifiable_auto<T, P, B, const WIDTH: usize, const HEIGHT: usize>(
    image_buffer: &ImageBuffer<P, Vec<P::Subpixel>>,
    collection: &mut VisCollection<T, P, WIDTH, HEIGHT>,
    builder: &B,
) -> Result<GridMap2D<T>, VisError<WIDTH, HEIGHT>>
where
    T: IdentifiableTile,
    P: DefaultPixel + 'static,
    B: IdentTileBuilder<T>,
{
    let size = check_grid_vis_size(image_buffer)?;
    let mut grid = GridMap2D::<T>::new(size);

    for position in size.get_all_possible_positions() {
        let pixels = VisCollection::<T, P, WIDTH, HEIGHT>::read_pixels_for_tile_at_pos(
            image_buffer,
            position,
        )?;
        let tile = builder.create_identifiable_tile(position, create_tile_id_from_pixels(&pixels));
        match collection.add_tile_pixels(&tile, image_buffer)? {
            super::collection::VisCollectionOutcome::Empty => {
                continue;
            }
            _ => grid.insert_tile(tile),
        };
    }
    Ok(grid)
}

/// Load [`GridMap2D`] of [`IdentifiableTile`] struct, using registered pixels in [`VisCollection`].
///
/// If you don't need to declare the `tile_id` yourself, you can use [`load_gridmap_identifiable_auto`], which generates
/// tile ids on basis of each pixel.
///
/// # Arguments
/// - `image_buffer` - an [ImageBuffer] containing the source image data.
/// - `collection` - a [`VisCollection`] containing the tile pixels. It needs to be populated before usage.
/// - `builder` - a struct which can be used to construct new tiles on basis of their `tile_id`. One of [`IdentTileBuilder`]
/// implementing objects.
pub fn load_gridmap_identifiable_manual<T, P, B, const WIDTH: usize, const HEIGHT: usize>(
    image_buffer: &ImageBuffer<P, Vec<P::Subpixel>>,
    collection: &VisCollection<T, P, WIDTH, HEIGHT>,
    builder: &B,
) -> Result<GridMap2D<T>, VisError<WIDTH, HEIGHT>>
where
    T: IdentifiableTile,
    P: DefaultPixel + 'static,
    B: IdentTileBuilder<T>,
{
    let size = check_grid_vis_size(image_buffer)?;
    let mut grid = GridMap2D::<T>::new(size);

    for position in size.get_all_possible_positions() {
        let pixels = VisCollection::<T, P, WIDTH, HEIGHT>::read_pixels_for_tile_at_pos(
            image_buffer,
            position,
        )?;
        if let Some(tile_id) = collection.get_tile_id_by_pixels(&pixels) {
            grid.insert_tile(builder.create_identifiable_tile(position, *tile_id));
        } else if !collection.is_empty(&pixels) {
            return Err(VisError::new_nonexist(position));
        }
    }
    Ok(grid)
}

pub fn init_map_image_buffer<P, const WIDTH: usize, const HEIGHT: usize>(
    grid_size: &GridSize,
) -> ImageBuffer<P, Vec<P::Subpixel>>
where
    P: DefaultPixel,
{
    ImageBuffer::new(grid_size.x() * WIDTH as u32, grid_size.y() * HEIGHT as u32)
}

pub fn write_gridmap_identifiable<T, P, const WIDTH: usize, const HEIGHT: usize>(
    image_buffer: &mut ImageBuffer<P, Vec<P::Subpixel>>,
    grid_map: &GridMap2D<T>,
    collection: &VisCollection<T, P, WIDTH, HEIGHT>,
) -> Result<(), VisError<WIDTH, HEIGHT>>
where
    T: IdentifiableTile,
    P: DefaultPixel + 'static,
{
    check_grid_image_size(image_buffer, grid_map.size())?;

    collection.draw_map(grid_map, image_buffer)?;

    Ok(())
}

pub fn write_gridmap_vis<T, P, const WIDTH: usize, const HEIGHT: usize>(
    image_buffer: &mut ImageBuffer<P, Vec<P::Subpixel>>,
    grid_map: &GridMap2D<T>,
) -> Result<(), VisError<WIDTH, HEIGHT>>
where
    T: VisTile2D<P, WIDTH, HEIGHT>,
    P: DefaultPixel + 'static,
{
    check_grid_image_size(image_buffer, grid_map.size())?;

    for position in grid_map.get_all_positions() {
        write_tile(
            image_buffer,
            position,
            &grid_map
                .get_tile_at_position(&position)
                .expect("cannot get tile!")
                .vis_pixels(),
        )?;
    }

    Ok(())
}

// ------ PRIVATE ------ //

pub(crate) fn create_tile_id_from_pixels<
    P: DefaultPixel,
    const WIDTH: usize,
    const HEIGHT: usize,
>(
    pixels: &[[P; WIDTH]; HEIGHT],
) -> u64 {
    let mut hasher = DefaultHasher::default();
    pixels.hash(&mut hasher);
    hasher.finish()
}

fn check_grid_vis_size<P: Pixel + 'static, const WIDTH: usize, const HEIGHT: usize>(
    image: &ImageBuffer<P, Vec<P::Subpixel>>,
) -> Result<GridSize, VisError<WIDTH, HEIGHT>> {
    if image.height() as usize % HEIGHT != 0 || image.width() as usize % WIDTH != 0 {
        Err(VisError::new_grid_load(image.width(), image.height()))
    } else {
        Ok(GridSize::new(
            image.width() / WIDTH as u32,
            image.height() / HEIGHT as u32,
        ))
    }
}

fn check_grid_image_size<P: Pixel + 'static, const WIDTH: usize, const HEIGHT: usize>(
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

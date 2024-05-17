//! [`IdentifiableTile`] instance builders.
//!
//! Many methods in `grid_forge` needs to construct new instances of [`GridMap2D`](crate::map::GridMap2D) and fill them with
//! new instances of tiles by their `tile_id`. For them to be flexible, they need to use a builder struct, using some strategy
//! to create new tile instances.
//!
//! User can create their own [`IdentTileBuilder`]-implementing struct to use their own method of building new tiles, though
//! there are already some builders provided, using some basic strategies.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::Display;
use std::marker::PhantomData;

use crate::tile::identifiable::IdentifiableTileData;
use crate::tile::{GridPosition, GridTile};

/// [`IdentTileBuilder`] which creates new tiles of [`Clone`]-implementing tile struct. Prototype of tile with each `tile_id` need to be
/// provided to the builder via [`add_tiles`](Self::add_tiles).
///
/// # Examples
/// ```
/// use grid_forge::GridPos2D;
/// # use grid_forge::tile::GridTile2D;
/// # use grid_forge::tile::identifiable::IdentifiableTile;
/// use grid_forge::tile::identifiable::builders::{ConstructableViaIdentifierTile, IdentTileBuilder, IdentTileCloneBuilder};
///
/// // Tile struct implementing `GridTile2D` and `IdentifiableTile`
/// #[derive(Clone)]
/// struct MyTile {
///     pos: GridPos2D,
///     tile_id: u64,
///     string: String
/// }
/// #
/// # impl GridTile2D for MyTile {
/// #     fn set_grid_position(&mut self, position: GridPos2D) {
/// #         self.pos = position;
/// #     }
/// #
/// #     fn grid_position(&self) -> GridPos2D {
/// #         self.pos
/// #     }
/// # }
/// #
/// # impl IdentifiableTile for MyTile {
/// #     fn tile_type_id(&self) -> u64 {
/// #         self.tile_id
/// #     }
/// # }
///
/// let mut builder = IdentTileCloneBuilder::<MyTile>::default();
/// let tiles = vec![
///     MyTile { pos: (0, 0), tile_id: 1, string: "First".to_string() },
///     MyTile { pos: (0, 0), tile_id: 2, string: "Second".to_string() },
/// ];
///
/// builder.add_tiles(&tiles, false);
///
/// if let Err(err) = builder.check_missing_ids(&[1,2,3]) {
///     assert_eq!(&[3], err.get_missing_tile_type_ids());
/// } else {
///     panic!("Should return error!");
/// }
///
/// let tile_1st = builder.build_tile_unchecked((2,3), 1);
/// assert_eq!(((2,3), 1, "First".to_string()), (tile_1st.pos, tile_1st.tile_id, tile_1st.string));
///
/// let tile_2nd = builder.build_tile_unchecked((3,4), 2);
/// assert_eq!(((3,4), 2, "Second".to_string()), (tile_2nd.pos, tile_2nd.tile_id, tile_2nd.string));
/// ```
#[derive(Debug, Clone)]
pub struct IdentTileCloneBuilder<Data: IdentifiableTileData + Clone> {
    tiles: BTreeMap<u64, Data>,
}

impl<T: IdentifiableTileData + Clone> Default for IdentTileCloneBuilder<T> {
    fn default() -> Self {
        Self {
            tiles: BTreeMap::new(),
        }
    }
}

impl<Data: IdentifiableTileData + Clone> IdentTileCloneBuilder<Data> {
    /// Provide tile prototypes to the builder, which will be used to create new tile instances.
    ///
    /// If `overwrite` is `true`, then if prototype for given `tile_id` has been already saved, it will be overwritten.
    pub fn add_tiles(&mut self, tiles: &[Data], overwrite: bool) {
        for tile in tiles {
            if !overwrite && self.tiles.contains_key(&tile.tile_type_id()) {
                continue;
            }
            self.tiles.insert(tile.tile_type_id(), tile.clone());
        }
    }
}

impl<Data: IdentifiableTileData + Clone> IdentTileBuilder<Data> for IdentTileCloneBuilder<Data> {
    fn build_tile_unchecked(&self, position: GridPosition, tile_type_id: u64) -> GridTile<Data> {
        let tile_data = self
            .tiles
            .get(&tile_type_id)
            .unwrap_or_else(|| panic!("can't get tile_data with `tile_type_id`: {tile_type_id}"))
            .clone();

        GridTile::new(position, tile_data)
    }

    fn build_tile(
        &self,
        position: GridPosition,
        tile_type_id: u64,
    ) -> Result<GridTile<Data>, TileBuilderError> {
        if let Some(tile) = self.tiles.get(&tile_type_id) {
            let data = tile.clone();
            Ok(GridTile::new(position, data))
        } else {
            Err(TileBuilderError::new(&[tile_type_id]))
        }
    }

    fn check_missing_ids(&self, tile_type_ids: &[u64]) -> Result<(), TileBuilderError> {
        let missing_ids = tile_type_ids
            .iter()
            .filter(|tile_id| !self.tiles.contains_key(tile_id))
            .copied()
            .collect::<Vec<_>>();

        if !missing_ids.is_empty() {
            Err(TileBuilderError::new(&missing_ids))
        } else {
            Ok(())
        }
    }
}

/// [`IdentTileBuilder`] which creates new tiles with given identifier based on the contructor functions provided to the
/// to the builder via [`set_tile_constructor`](Self::set_tile_constructor).
///
/// # Examples
/// ```
/// use grid_forge::GridPos2D;
/// # use grid_forge::tile::GridTile2D;
/// # use grid_forge::tile::identifiable::IdentifiableTile;
/// use grid_forge::tile::identifiable::builders::{ConstructableViaIdentifierTile, IdentTileBuilder, IdentTileFunBuilder};
///
/// // Tile struct implementing `GridTile2D` and `IdentifiableTile`
/// struct MyTile {
///     pos: GridPos2D,
///     tile_id: u64,
///     traversible: bool
/// }
/// #
/// # impl GridTile2D for MyTile {
/// #     fn set_grid_position(&mut self, position: GridPos2D) {
/// #         self.pos = position;
/// #     }
/// #
/// #     fn grid_position(&self) -> GridPos2D {
/// #         self.pos
/// #     }
/// # }
/// #
/// # impl IdentifiableTile for MyTile {
/// #     fn tile_type_id(&self) -> u64 {
/// #         self.tile_id
/// #     }
/// # }
///
/// let mut builder = IdentTileFunBuilder::<MyTile>::default();
/// builder.set_tile_constructor(1, |pos: GridPos2D, tile_id: u64|( MyTile { pos, tile_id, traversible: true} ));
/// builder.set_tile_constructor(2, |pos: GridPos2D, tile_id: u64|( MyTile { pos, tile_id, traversible: false} ));
///
/// if let Err(err) = builder.check_missing_ids(&[1,2,3]) {
///     assert_eq!(&[3], err.get_missing_tile_type_ids());
/// } else {
///     panic!("Should return error!");
/// }
///
/// let tile_1st = builder.build_tile_unchecked((2,3), 1);
/// assert_eq!(((2,3), 1, true), (tile_1st.pos, tile_1st.tile_id, tile_1st.traversible));
///
/// let tile_2nd = builder.build_tile_unchecked((3,4), 2);
/// assert_eq!(((3,4), 2, false), (tile_2nd.pos, tile_2nd.tile_id, tile_2nd.traversible));
/// ```
#[derive(Debug, Clone)]
pub struct IdentTileFunBuilder<T: IdentifiableTileData> {
    funs: BTreeMap<u64, fn(u64) -> T>,
}

impl<Data: IdentifiableTileData> IdentTileFunBuilder<Data> {
    pub fn set_tile_constructor(&mut self, tile_id: u64, constructor: fn(u64) -> Data) {
        self.funs.insert(tile_id, constructor);
    }

    pub fn clear(&mut self) {
        self.funs.clear();
    }
}

impl<Data: IdentifiableTileData> Default for IdentTileFunBuilder<Data> {
    fn default() -> Self {
        Self {
            funs: BTreeMap::new(),
        }
    }
}

impl<Data: IdentifiableTileData> IdentTileBuilder<Data> for IdentTileFunBuilder<Data> {
    fn build_tile_unchecked(&self, position: GridPosition, tile_type_id: u64) -> GridTile<Data> {
        let fun = self.funs.get(&tile_type_id).unwrap_or_else(|| {
            panic!("can't get tile function with `tile_type_id`: {tile_type_id}")
        });

        GridTile::new(position, fun(tile_type_id))
    }

    fn build_tile(
        &self,
        position: GridPosition,
        tile_id: u64,
    ) -> Result<GridTile<Data>, TileBuilderError> {
        if let Some(fun) = self.funs.get(&tile_id) {
            Ok(GridTile::new(position, fun(tile_id)))
        } else {
            Err(TileBuilderError::new(&[tile_id]))
        }
    }

    fn check_missing_ids(&self, tile_ids: &[u64]) -> Result<(), TileBuilderError> {
        let missing_ids = tile_ids
            .iter()
            .filter(|tile_id| !self.funs.contains_key(tile_id))
            .copied()
            .collect::<Vec<_>>();

        if !missing_ids.is_empty() {
            Err(TileBuilderError::new(&missing_ids))
        } else {
            Ok(())
        }
    }
}

/// Trait which allows creating new istance of struct implementing [`IdentifiableTile`].
///
/// See also [`BasicIdentifiableTile2D`](crate::tile::identifiable::BasicIdentifiableTile2D) for basic identifiable tile type,
/// implementing this trait out of the box, for applications where you don't need your tile struct to hold any additional information.
///
/// # Examples
///
/// Implementing this trait for you custom tile makes it constructable via [`IdentTileTraitBuilder`].
///
/// ```
/// use grid_forge::GridPos2D;
/// # use grid_forge::tile::GridTile2D;
/// # use grid_forge::tile::identifiable::IdentifiableTile;
/// use grid_forge::tile::identifiable::builders::{ConstructableViaIdentifierTile, IdentTileBuilder, IdentTileTraitBuilder};
///
/// // Tile struct implementing `GridTile2D` and `IdentifiableTile`
/// struct MyTile {
///     pos: GridPos2D,
///     tile_id: u64
/// }
///
/// # impl GridTile2D for MyTile {
/// #     fn set_grid_position(&mut self, position: GridPos2D) {
/// #         self.pos = position;
/// #     }
/// #
/// #     fn grid_position(&self) -> GridPos2D {
/// #         self.pos
/// #     }
/// # }
/// #
/// # impl IdentifiableTile for MyTile {
/// #     fn tile_type_id(&self) -> u64 {
/// #         self.tile_id
/// #     }
/// # }
/// impl ConstructableViaIdentifierTile for MyTile {
///     fn tile_new(pos: GridPos2D, tile_id: u64) -> Self {
///         Self { pos, tile_id }
///     }
/// }
///
/// let builder = IdentTileTraitBuilder::<MyTile>::default();
/// let tile = builder.build_tile_unchecked((2,3), 45);
///
/// assert_eq!((2,3), tile.grid_position());
/// assert_eq!(45, tile.tile_type_id());
/// ```
pub trait ConstructableViaIdentifierTile
where
    Self: IdentifiableTileData,
{
    fn tile_new(position: GridPosition, tile_type_id: u64) -> GridTile<Self>;
}

/// [`IdentTileBuilder`] which creates new tiles with given identifier based on the tile implementation of
/// [`ConstructableViaIdentifierTile`]. No need to add any tile creators.
///
/// [`IdentTileBuilder::check_missing_ids`] is an no-op for this struct.
///
/// Refer to documentation of [`ConstructableViaIdentifierTile`] for usage example.
#[derive(Debug, Clone)]
pub struct IdentTileTraitBuilder<Data: IdentifiableTileData + ConstructableViaIdentifierTile> {
    phantom: PhantomData<Data>,
}

impl<Data: IdentifiableTileData + ConstructableViaIdentifierTile> Default
    for IdentTileTraitBuilder<Data>
{
    fn default() -> Self {
        Self {
            phantom: PhantomData::<Data>,
        }
    }
}

impl<Data: IdentifiableTileData + ConstructableViaIdentifierTile> IdentTileBuilder<Data>
    for IdentTileTraitBuilder<Data>
{
    fn build_tile_unchecked(&self, position: GridPosition, tile_type_id: u64) -> GridTile<Data> {
        Data::tile_new(position, tile_type_id)
    }

    fn build_tile(
        &self,
        position: GridPosition,
        tile_type_id: u64,
    ) -> Result<GridTile<Data>, TileBuilderError> {
        Ok(Data::tile_new(position, tile_type_id))
    }

    fn check_missing_ids(&self, _tile_ids: &[u64]) -> Result<(), TileBuilderError> {
        Ok(())
    }
}

/// Trait shared by objects which on basis of the grid position and tile identifier of given [`IdentifiableTile`]-implementing struct can
/// create correct instance of the tile. Necessary for many [`GridMap2D`](crate::map::GridMap2D) creating methods.
///
/// Three different builders are available in the `grid_forge`:
/// - [`IdentTileFunBuilder`] - for tiles not implementing any additional traits.
/// - [`IdentTileCloneBuilder`] - for tiles implementing [`Clone`].
/// - [`IdentTileTraitBuilder`] - for tiles implementing [`ConstructableViaIdentifierTile`].
pub trait IdentTileBuilder<Data: IdentifiableTileData> {
    /// Creates tile with given tile identifier at given grid position.
    ///
    /// # Panics
    /// Can panic if builder does not have possibility to construct tile of given `tile_id` based on the gathered information. You can check
    /// for missing tile ids with [`check_missing_ids`](IdentTileBuilder::check_missing_ids) or use its fallible version:
    /// [`build_tile`](IdentTileBuilder::build_tile).
    fn build_tile_unchecked(&self, position: GridPosition, tile_type_id: u64) -> GridTile<Data>;

    /// Creates tile with given tile identifier at given grid position. Returns error if cannot construct tile of given `tile_id`.
    fn build_tile(
        &self,
        position: GridPosition,
        tile_type_id: u64,
    ) -> Result<GridTile<Data>, TileBuilderError>;

    /// Checks for missing tile creators out of provided slice of `tile_id`.
    fn check_missing_ids(&self, tile_type_ids: &[u64]) -> Result<(), TileBuilderError>;
}

/// Error stemming from missing tiles in [`IdentTileBuilder`].
#[derive(Debug, Clone)]
pub struct TileBuilderError {
    tile_type_ids: Vec<u64>,
}

impl TileBuilderError {
    fn new(tile_ids: &[u64]) -> Self {
        Self {
            tile_type_ids: Vec::from(tile_ids),
        }
    }

    pub fn get_missing_tile_type_ids(&self) -> &[u64] {
        &self.tile_type_ids
    }
}

impl Display for TileBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "missing tile ids from builder: {missing_ids}",
            missing_ids = self
                .tile_type_ids
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl Error for TileBuilderError {}

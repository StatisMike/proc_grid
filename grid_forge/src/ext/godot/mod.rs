pub mod collection;
pub mod error;
pub mod ops;

#[derive(Clone, Copy)]
pub(crate) enum TileSourceType {
    Atlas,
    Collection,
    Mesh,
}

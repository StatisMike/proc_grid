use crate::tile::identifiable::IdentifiableTileData;

use super::CollapsibleTileData;

pub trait CollapsibleResolverSubscriber {
    fn on_collapse<Ref>(&mut self, collapsed: &Ref)
    where
        Ref: AsRef<CollapsibleTileData>;
}

#[cfg(feature = "vis")]
mod vis {
    use std::sync::Mutex;

    use image::{ImageBuffer, Pixel};
    pub struct VisWriteSubscriber<P, const WIDTH: usize, const HEIGHT: usize>
    where
        P: Pixel,
    {
        image_buffer: Mutex<ImageBuffer<P, Vec<P::Subpixel>>>,
    }
}

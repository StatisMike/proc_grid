use std::marker::PhantomData;

use crate::tile::identifiable::IdentifiableTile;

pub mod adjacency;

pub struct AdjacencyRuleset<InputTile>
where
    InputTile: IdentifiableTile,
{
    input_tile_type: PhantomData<InputTile>,
}

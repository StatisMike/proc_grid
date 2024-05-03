use std::marker::PhantomData;

use crate::tile::identifiable::IdentifiableTile;

use self::adjacency::AdjacencyRules;

pub mod adjacency;

pub struct AdjacencyRuleset<InputTile>
where
    InputTile: IdentifiableTile,
{
    input_tile_type: PhantomData<InputTile>,
}

// pub struct CollapsibleRules<T> {
//     adjacency: AdjacencyRules<T>
// }
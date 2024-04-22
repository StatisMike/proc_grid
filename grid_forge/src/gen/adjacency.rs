use std::{collections::{BTreeSet, HashMap}, marker::PhantomData};

use crate::{map::GridDir, tile::GridTile2D};

/// Its implementation makes the specific tile identifiable and discernable from other tile instances in regards to tile 
/// type. For the generative algorithms using this trait to match and select tiles, general rules of the tile identity 
/// when implementing this trait manually should be:
/// 
/// - its position **shouldn't be ever taken into account**. Tile of these same type could be placed on different positions 
/// on the GridMap.
/// - other properties of the tile (such as visual representation) *can* be taken into account depending on your specific 
/// needs.  
/// 
/// Used in algorithms:
/// - Wafe Function Collapse ([WFC](crate::gen::wfc)) 
pub trait IdentifiableTile 
where Self: GridTile2D
{
  fn get_tile_id(&self) -> u64;
}

#[derive(Debug, Clone)]
pub struct AdjacencyRules<T>
where T: IdentifiableTile
{
  inner: HashMap<u64, InnerAdjacency>,
  id_type: PhantomData<T>
}

impl<T> Default for AdjacencyRules<T>
where T: IdentifiableTile
{
    fn default() -> Self {
        Self { inner: HashMap::new(), id_type: PhantomData::<T> }
    }
}

impl<T> AdjacencyRules<T>
where T: IdentifiableTile
{
  pub fn add_adjacency(&mut self, tile: &T, adjacent_tile: &T, direction: GridDir) {
    self.add_adjacency_raw(tile.get_tile_id(), adjacent_tile.get_tile_id(), direction)
    
  }

  pub(crate) fn add_adjacency_raw(&mut self, tile_id: u64, adjacent_id: u64, direction: GridDir) {
    let adjacents = self.inner.entry(tile_id).or_default();

    adjacents.add_option(adjacent_id, direction);
  }

  pub(crate) fn is_valid_raw(&self, tile_id: u64, adjacent_id: u64, direction: GridDir) -> bool {
    self.inner.get(&tile_id).unwrap().is_in_options(adjacent_id, direction)
  }
}

#[derive(Debug, Clone)]
struct InnerAdjacency {
  ia: HashMap<GridDir, BTreeSet<u64>>
}

impl Default for InnerAdjacency {
    fn default() -> Self {
      let mut ia = HashMap::new();
      for dir in GridDir::ALL {
        ia.insert(*dir, BTreeSet::new());
      }
        Self { ia }
    }
}

impl InnerAdjacency {
  fn add_option(&mut self, adjacent_id: u64, dir: GridDir) {
    let opts = self.ia.get_mut(&dir).unwrap();
    opts.insert(adjacent_id);
  }

  fn is_in_options(&self, adjacent_id: u64, dir: GridDir) -> bool
  {
    self.ia.get(&dir).unwrap().contains(&adjacent_id)
  }
}
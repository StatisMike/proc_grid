use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::hash::{DefaultHasher, Hash, Hasher};

/// Trait implementing the behaviour for easily keeping and retrieving the `tile_type_id` alongside some arbitrary [`DATA`](IdentTileCollection::DATA)
///  kept for the tile.
///
/// Objects implementing this trait are often used for operations of transferring between non-`grid-forge` representation of grid map and  
/// [`GridMap2D`](crate::map::GridMap2D) alongside builders implementing [`IdentTileBuilder`](crate::tile::identifiable::builders::IdentTileBuilder)
/// trait.
///
/// For the collection to work properly, both `tile_type_id` needs to be unique for tile of each type (as required by
/// [`identifiable`](crate::tile::identifiable) module assumptions), as well as the [`DATA`](IdentTileCollection::DATA) itself.
///
/// ## Automatic derivement of `tile_type_id`
///
/// As the external representations rarely hold a specific `tile_type_id`, the collection handles storing the `DATA` without specifying the
/// identifier manually (see: [`add_tile`](IdentTileCollection::add_tile) and [`set_tile`](IdentTileCollection::set_tile)). It calculates
/// the `type_tile_id` by hashing the underlying data using [`DefaultHasher`].
///
/// This makes the collection instantly useful, but only during transfers between one external representation and the `GridMap2D`, as
/// synchronizing the `tile_type_id` between multiple collections is very difficult.
///
/// ## Manual specification of `tile_type_id`
///
/// To synchronize between multiple external representations of grid map, with `GridMap2D` working as an intermediate step, the `DATA`
/// should be stored in controlled way in multiple `IdentTileCollection`s to keep the `tile_type_id` synchronized (see:
/// [`add_tile_data`](IdentTileCollection::add_tile_data) and [`set_tile_data`](IdentTileCollection::add_tile_data)).
///
/// ## Prefer specific facade methods
///
/// Methods of this trait are intended to be generic and low-level. Specific implementors can wrap them below additional logic. If these
/// kind of methods are available, prefer to make use of them.
pub trait IdentTileCollection {
    /// External data kept within the collection, bind to the specific `tile_type_id`.
    type DATA: Hash;

    /// Exposes inner hashmap. Necessary for other methods implementations.
    fn inner(&self) -> &HashMap<u64, Self::DATA>;
    /// Exposes inner hashmap mutably. Necessary for other methods implementations.
    fn inner_mut(&mut self) -> &mut HashMap<u64, Self::DATA>;
    /// Exposes reverse hashmap. Necessary for other methods implementations.
    fn rev(&self) -> &HashMap<u64, u64>;
    /// Exposes reverse hashmap mutably. Necessary for other methods implementations.
    fn rev_mut(&mut self) -> &mut HashMap<u64, u64>;

    /// Adds tile data without `tile_type_id` provided - it will be generated with [get_data_hash](IdentTileCollection::get_data_hash).
    /// If either data or the generated `tile_type_id` are already present in the collection, addition will be skipped, returning `false`.
    fn add_tile(&mut self, data: Self::DATA) -> bool {
        let hash = Self::generate_type_id(&data);
        if self.rev().contains_key(&hash) {
            return false;
        }
        self.add_tile_data(hash, data)
    }

    /// Sets tile data without `tile_type_id` provided - it will be generated with [get_data_hash](IdentTileCollection::get_data_hash).
    /// If either data or generated `tile_type_id` are already present in the collection, they will be overwritten, returning `true`.
    fn set_tile(&mut self, data: Self::DATA) -> bool {
        let hash = Self::generate_type_id(&data);
        if let Some(type_id) = self.rev_mut().remove(&hash) {
            self.inner_mut().remove(&type_id);
        }
        self.set_tile_data(hash, data).is_some()
    }

    /// Removes tile data if either provided data or hash generated with [get_data_hash](IdentTileCollection::get_data_hash) is present
    /// in the collection.
    fn remove_tile(&mut self, data: Self::DATA) -> bool {
        let mut changed = false;
        let hash = Self::generate_type_id(&data);
        if let Some(type_id) = self.rev_mut().remove(&hash) {
            self.inner_mut().remove(&type_id);
            changed = true;
        }
        if self.remove_tile_data(hash).is_some() {
            changed = true;
        }
        changed
    }

    /// Adds `data` for specified `tile_type_id`. If given `tile_type_id` is already present, no changes are made and returns `false`;
    fn add_tile_data(&mut self, tile_type_id: u64, data: Self::DATA) -> bool {
        let changed = if let Entry::Vacant(e) = self.inner_mut().entry(tile_type_id) {
            let data_hash = Self::generate_type_id(&data);
            e.insert(data);
            Some(data_hash)
        } else {
            None
        };
        if let Some(data_hash) = changed {
            self.rev_mut().insert(data_hash, tile_type_id);
        }
        changed.is_some()
    }

    /// Sets `data` for specified `tile_type_id`. Returns removed data stored for specified id, if present.
    fn set_tile_data(&mut self, tile_type_id: u64, data: Self::DATA) -> Option<Self::DATA> {
        let data_hash = Self::generate_type_id(&data);
        let existing_data = self.inner_mut().insert(tile_type_id, data);
        let existing_hash = existing_data.as_ref().map(|data| Self::generate_type_id(data));
        if let Some(hash_to_remove) = existing_hash {
            self.rev_mut().remove(&hash_to_remove);
        }
        self.rev_mut().insert(data_hash, tile_type_id);

        existing_data
    }

    /// Removes [`DATA`](IdentTileCollection::DATA) for provided `tile_type_id`. Returns removed data.
    fn remove_tile_data(&mut self, tile_type_id: u64) -> Option<Self::DATA> {
        let existing_data = self.inner_mut().remove(&tile_type_id);
        if let Some(data_to_remove) = &existing_data {
            self.inner_mut()
                .remove(&Self::generate_type_id(data_to_remove));
        }
        existing_data
    }

    /// Gets [`DATA`](IdentTileCollection::DATA) stored for given `tile_type_id`.
    fn get_tile_data(&self, tile_type_id: u64) -> Option<&Self::DATA> {
        self.inner().get(&tile_type_id)
    }

    /// Gets `tile_type_id` bind to given `data`.
    fn get_tile_type_id(&self, data: &Self::DATA) -> Option<u64> {
        self.rev().get(&Self::generate_type_id(data)).copied()
    }

    /// Generates `tile_type_id` using provided [`DATA`](IdentTileCollection::DATA).
    fn generate_type_id(data: &Self::DATA) -> u64 {
        let mut hasher = DefaultHasher::default();
        data.hash(&mut hasher);
        hasher.finish()
    }
}

#[cfg(test)]
mod test {
    use super::IdentTileCollection;
    use std::collections::HashMap;

    struct TestTileCollection {
        inner: HashMap<u64, i32>,
        rev: HashMap<u64, u64>,
    }

    impl IdentTileCollection for TestTileCollection {
        type DATA = i32;

        fn inner(&self) -> &HashMap<u64, Self::DATA> {
            &self.inner
        }

        fn inner_mut(&mut self) -> &mut HashMap<u64, Self::DATA> {
            &mut self.inner
        }

        fn rev(&self) -> &HashMap<u64, u64> {
            &self.rev
        }

        fn rev_mut(&mut self) -> &mut HashMap<u64, u64> {
            &mut self.rev
        }
    }
}

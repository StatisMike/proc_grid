use std::collections::HashMap;
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

    /// Additional operation that will be done during addition of new `DATA` into the
    /// collection. No-op by default.
    fn on_add(&mut self, _data: &Self::DATA) {}

    /// Additional operation that will be done during removal of old `DATA` from the
    /// collection. No-op by default.
    fn on_remove(&mut self, _data: &Self::DATA) {}

    /// Adds tile data without `tile_type_id` provided - it will be generated with [generate_type_id](IdentTileCollection::generate_type_id).
    /// If either data or the generated `tile_type_id` are already present in the collection, addition will be skipped, returning `false`.
    fn add_tile(&mut self, data: Self::DATA) -> bool {
        let hash = Self::generate_type_id(&data);
        if self.rev().contains_key(&hash) {
            return false;
        }
        self.add_tile_data(hash, data)
    }

    /// Sets tile data without `tile_type_id` provided - it will be generated with [generate_type_id](IdentTileCollection::generate_type_id).
    /// If either data or generated `tile_type_id` are already present in the collection, they will be overwritten, returning `true`.
    fn set_tile(&mut self, data: Self::DATA) -> bool {
        let mut changed = false;
        let hash = Self::generate_type_id(&data);
        if let Some(type_id) = self.rev_mut().remove(&hash) {
            if let Some(existing_data) = self.inner_mut().remove(&type_id) {
                self.on_remove(&existing_data);
            }
            changed = true;
        }
        if self.set_tile_data(hash, data).is_some() {
            changed = true;
        }
        changed
    }

    /// Removes tile data if either provided data or hash generated with [generate_type_id](IdentTileCollection::generate_type_id) is present
    /// in the collection.
    fn remove_tile(&mut self, data: &Self::DATA) -> bool {
        let mut changed = false;
        let hash = Self::generate_type_id(data);
        if let Some(type_id) = self.rev_mut().remove(&hash) {
            if let Some(removed_data) = self.inner_mut().remove(&type_id) {
                self.on_remove(&removed_data);
            }
            changed = true;
        }
        if self.remove_tile_data(&hash).is_some() {
            changed = true;
        }
        changed
    }

    /// Adds `data` for specified `tile_type_id`. If given `tile_type_id` is already present, no changes are made and returns `false`;
    fn add_tile_data(&mut self, tile_type_id: u64, data: Self::DATA) -> bool {
        if self.inner().contains_key(&tile_type_id) {
            return false;
        }
        self.on_add(&data);
        let data_hash = Self::generate_type_id(&data);
        self.inner_mut().insert(tile_type_id, data);
        self.rev_mut().insert(data_hash, tile_type_id);
        true
    }

    /// Sets `data` for specified `tile_type_id`. Returns removed data stored for specified id, if present.
    fn set_tile_data(&mut self, tile_type_id: u64, data: Self::DATA) -> Option<Self::DATA> {
        let data_hash = Self::generate_type_id(&data);
        self.on_add(&data);
        let existing_data = self.inner_mut().insert(tile_type_id, data);
        let existing_hash = existing_data
            .as_ref()
            .map(|data| Self::generate_type_id(data));
        if let Some(hash_to_remove) = existing_hash {
            self.on_remove(existing_data.as_ref().unwrap());
            self.rev_mut().remove(&hash_to_remove);
        }
        self.rev_mut().insert(data_hash, tile_type_id);

        existing_data
    }

    /// Removes [`DATA`](IdentTileCollection::DATA) for provided `tile_type_id`. Returns removed data.
    fn remove_tile_data(&mut self, tile_type_id: &u64) -> Option<Self::DATA> {
        let existing_data = self.inner_mut().remove(tile_type_id);
        if let Some(data_to_remove) = &existing_data {
            self.on_remove(data_to_remove);
            self.remove_tile(data_to_remove);
        }
        existing_data
    }

    /// Gets [`DATA`](IdentTileCollection::DATA) stored for given `tile_type_id`.
    fn get_tile_data(&self, tile_type_id: &u64) -> Option<&Self::DATA> {
        self.inner().get(tile_type_id)
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

    #[derive(Default, Clone)]
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

    const TEST_DATA: [i32; 7] = [34231, 1223, -1943, -12453, i32::MAX, i32::MIN, 0];

    #[test]
    fn test_collection_data_only() {
        let mut collection = TestTileCollection::default();

        for data in TEST_DATA {
            assert!(collection.add_tile(data), "no data: {data} has been added");
        }

        assert_eq!(TEST_DATA.len(), collection.inner().len());
        assert_eq!(TEST_DATA.len(), collection.rev().len());

        for data in TEST_DATA {
            assert!(!collection.add_tile(data), "data: {data} has been added");
        }

        assert_eq!(TEST_DATA.len(), collection.inner().len());
        assert_eq!(TEST_DATA.len(), collection.rev().len());

        for data in TEST_DATA {
            assert!(collection.set_tile(data), "data: {data} has not been set");
        }

        assert_eq!(TEST_DATA.len(), collection.inner().len());
        assert_eq!(TEST_DATA.len(), collection.rev().len());

        let mut tile_type_ids = Vec::new();

        for data in TEST_DATA {
            if let Some(type_id) = collection.get_tile_type_id(&data) {
                tile_type_ids.push(type_id);
            } else {
                panic!("didn't extract tile type id for tile data: {data}");
            }
        }

        for (data_id, data) in TEST_DATA.iter().enumerate() {
            let extracted_data = collection
                .get_tile_data(tile_type_ids.get(data_id).expect("cannot get tile type id"))
                .expect("cannot get tile data");
            assert_eq!(data, extracted_data);
        }

        for (iter, data) in TEST_DATA.iter().enumerate() {
            collection.remove_tile(data);
            assert_eq!(TEST_DATA.len() - (iter + 1), collection.inner().len());
            assert_eq!(TEST_DATA.len() - (iter + 1), collection.rev().len());
        }
    }

    #[test]
    fn test_collection_data_by_id() {
        let mut collection = TestTileCollection::default();

        for (tile_type_id, data) in TEST_DATA.iter().enumerate() {
            assert!(
                collection.add_tile_data(tile_type_id as u64, *data),
                "no data: {data} has been added"
            );
        }

        assert_eq!(TEST_DATA.len(), collection.inner().len());
        assert_eq!(TEST_DATA.len(), collection.rev().len());

        for (tile_type_id, data) in TEST_DATA.iter().enumerate() {
            assert!(
                !collection.add_tile_data(tile_type_id as u64, *data),
                "data: {data} has been added"
            );
        }

        assert_eq!(TEST_DATA.len(), collection.inner().len());
        assert_eq!(TEST_DATA.len(), collection.rev().len());

        for (tile_type_id, data) in TEST_DATA.iter().enumerate() {
            let existing_data = collection.set_tile_data(tile_type_id as u64, *data);
            assert!(existing_data.is_some(), "data: {data} has not been set");
            assert_eq!(*data, existing_data.unwrap());
        }

        assert_eq!(TEST_DATA.len(), collection.inner().len());
        assert_eq!(TEST_DATA.len(), collection.rev().len());

        for (data_id, data) in TEST_DATA.iter().enumerate() {
            let extracted_data = collection
                .get_tile_data(&(data_id as u64))
                .expect("cannot get tile data");
            assert_eq!(data, extracted_data);
        }

        for (iter, data) in TEST_DATA.iter().enumerate() {
            let removed_data = collection.remove_tile_data(&(iter as u64));
            assert!(removed_data.is_some());
            assert_eq!(*data, removed_data.unwrap());
            assert_eq!(
                TEST_DATA.len() - (iter + 1),
                collection.inner().len(),
                "unchanged inner"
            );
            assert_eq!(
                TEST_DATA.len() - (iter + 1),
                collection.rev().len(),
                "unchanged rev"
            );
        }
    }

    #[test]
    fn test_collection_mixed() {
        let mut collection = TestTileCollection::default();

        for (tile_type_id, data) in TEST_DATA.iter().enumerate() {
            assert!(
                collection.add_tile_data(tile_type_id as u64, *data),
                "no data: {data} has been added"
            );
        }

        for data_idx in [1, 3, 5] {
            assert!(
                collection.set_tile(TEST_DATA[data_idx]),
                "no data for idx: {data_idx} has been set"
            );
        }

        assert_eq!(
            TEST_DATA.len(),
            collection.inner().len(),
            "wrong `inner` length"
        );
        assert_eq!(
            TEST_DATA.len(),
            collection.rev().len(),
            "wrong `rev` length"
        );

        let mut tile_type_ids = Vec::new();

        for data in TEST_DATA {
            if let Some(type_id) = collection.get_tile_type_id(&data) {
                tile_type_ids.push(type_id);
            } else {
                panic!("didn't extract tile type id for tile data: {data}");
            }
        }

        for (data_id, data) in TEST_DATA.iter().enumerate() {
            let extracted_data = collection
                .get_tile_data(tile_type_ids.get(data_id).expect("cannot get tile type id"))
                .expect("cannot get tile data");
            assert_eq!(data, extracted_data);
        }
    }
}

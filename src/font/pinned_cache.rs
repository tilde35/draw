use crate::font::pinned_data::PinnedData;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::*;

/// The cache editor provides lock-free reads
pub struct CacheEditor<'a, Key, Value: 'static> {
    cache: &'a PinnedCache<Key, Value>,
    lookup: Arc<HashMap<Key, &'a Value>>,
}
impl<'a, Key: Clone + Eq + Hash, Value: 'static> CacheEditor<'a, Key, Value> {
    pub fn get(&self, key: &Key) -> Option<&'a Value> {
        self.lookup.get(key).map(|v| *v)
    }
    // pub fn refresh(&mut self) {
    //     self.lookup = self.cache.get_lookup();
    // }
    pub fn add_missing(&mut self, values: Vec<(Key, Value)>) {
        self.lookup = self.cache.add_missing_entries(values);
    }
    pub fn add(&mut self, key: Key, value: Value) -> &'a Value {
        self.add_missing(vec![(key.clone(), value)]);
        self.get(&key).unwrap()
    }
}
impl<'a, Key, Value: 'static> std::clone::Clone for CacheEditor<'a, Key, Value> {
    fn clone(&self) -> Self {
        CacheEditor {
            cache: self.cache,
            lookup: self.lookup.clone(),
        }
    }
}

struct PinnedCacheData<Key, Value: 'static> {
    cache: PinnedData<Value>,
    // IMPORTANT: There is no 'self lifetime, need to use 'static and be careful about how data is returned
    unsafe_lookup: Mutex<Arc<HashMap<Key, &'static Value>>>,
}

pub struct PinnedCache<Key, Value: 'static> {
    data: Arc<PinnedCacheData<Key, Value>>,
}
impl<Key: Clone + Eq + Hash, Value: 'static> PinnedCache<Key, Value> {
    pub fn for_page_size(page_size: usize) -> Self {
        Self {
            data: Arc::new(PinnedCacheData {
                cache: PinnedData::for_page_size(page_size),
                unsafe_lookup: Mutex::new(Arc::new(HashMap::new())),
            }),
        }
    }
    // fn get_lookup<'a>(&'a self) -> Arc<HashMap<Key, &'a Value>> {
    //     self.data.unsafe_lookup.lock().unwrap().clone()
    // }
    pub fn editor<'a>(&'a self) -> CacheEditor<'a, Key, Value> {
        let lookup = self.data.unsafe_lookup.lock().unwrap().clone();
        CacheEditor {
            cache: &self,
            lookup,
        }
    }
    // pub fn get<'a>(&'a self, key: &Key) -> Option<&'a Value> {
    //     self.data.unsafe_lookup.lock().unwrap().get(key).map(|v| *v)
    // }
    // pub fn add_missing(&self, values: Vec<(Key, Value)>) {
    //     self.add_missing_entries(values);
    // }
    fn add_missing_entries<'a>(
        &'a self,
        values: Vec<(Key, Value)>,
    ) -> Arc<HashMap<Key, &'a Value>> {
        let mut lock = self.data.unsafe_lookup.lock().unwrap();
        let prev_lookup: &HashMap<_, _> = lock.as_ref();

        let mut new_lookup: HashMap<_, _> = prev_lookup.clone();

        for (k, v) in values.into_iter() {
            if !new_lookup.contains_key(&k) {
                let data = self.data.cache.add(v);

                // Pretend this is static (since there is no 'self lifetime)
                let data_ptr: *const Value = data;
                new_lookup.insert(k, unsafe { &(*data_ptr) });
            }
        }

        let result = Arc::new(new_lookup);
        *lock = result.clone();
        result
    }
}

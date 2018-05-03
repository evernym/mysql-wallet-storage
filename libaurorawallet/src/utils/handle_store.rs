use std::sync::{RwLock, Arc};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};

pub struct HandleStore<T> {
    ids_counter: AtomicUsize,
    map: RwLock<HashMap<i32, Arc<T>>>,
}

impl<T> HandleStore<T> {
    pub fn new() -> Self {
        HandleStore{ids_counter: ATOMIC_USIZE_INIT, map: RwLock::new(HashMap::new())}
    }

    pub fn get(&self, handle: i32) -> Option<Arc<T>> {
        match self.map.read() {
            Err(_) => None,
            Ok(ref map) => match map.get(&handle) {
                None => None,
                Some(x) => Some(x.clone()),
            }
        }
    }

    pub fn insert(&self, storage: T) -> i32 {
        let handle = (self.ids_counter.fetch_add(1, Ordering::SeqCst) + 1) as i32;
        self.map.write().unwrap().insert(handle, Arc::new(storage));
        handle
    }

    pub fn remove(&self, handle: i32) -> bool {
        self.map.write().unwrap().remove(&handle).is_some()
    }
}
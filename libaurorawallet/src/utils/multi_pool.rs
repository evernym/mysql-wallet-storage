use std::sync::{RwLock, Arc};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use mysql::Pool;

pub struct MultiPool {
    map: RwLock<HashMap<String, Arc<Pool>>>,
}

impl MultiPool {
    pub fn new() -> Self {
        MultiPool{map: RwLock::new(HashMap::new())}
    }

    pub fn get(&self, connection_string: &str) -> Option<Arc<Pool>> {
        let mut c = match self.map.read() {
            Err(_) => None,
            Ok(ref map) => match map.get(connection_string) {
                None => None,
                Some(x) => Some(x.clone()),
            }
        };

        if c.is_none() {
            let pool = Pool::new_manual(1, 100, connection_string);
            if pool.is_err() {
                return None
            }
            let pool = Arc::new(pool.unwrap());
            self.map.write().unwrap().insert(connection_string.to_owned(), pool.clone());

            c = Some(pool)
        }

        c
    }
}
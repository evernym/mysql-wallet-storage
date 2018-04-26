use utils::handle_store::HandleStore;
use std::sync::Arc;
use mysql::Pool;
use utils::error_code::ErrorCode;

struct Record {
    id: String,
    value: Option<Vec<u8>>,
    tags: Option<String>,
    type_: Option<String>,
}

struct SearchRecord {
}

pub struct AuroraStorage {
    wallet_id: u64,
    records: HandleStore<Record>,
    searches: HandleStore<SearchRecord>,
//    read_pool: Arc<Pool>, // cached reference to the pool
//    write_pool: Arc<Pool>,
}

impl AuroraStorage {
    pub fn new(wallet_id: u64) -> Self {
        Self{wallet_id, records: HandleStore::new(), searches: HandleStore::new()}
    }

    pub fn free_record(&self, record_handle: i32) -> ErrorCode {
        if self.records.remove(record_handle) {
            ErrorCode::Success
        }
        else {
            ErrorCode::InvalidRecordHandle
        }
    }

    pub fn free_search(&self, search_handle: i32) -> ErrorCode {
        if self.searches.remove(search_handle) {
            ErrorCode::Success
        }
        else {
            ErrorCode::InvalidSearchHandle
        }
    }
}
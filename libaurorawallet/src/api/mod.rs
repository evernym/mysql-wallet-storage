use utils::handle_store::HandleStore;
use utils::error_code::ErrorCode;
use aurora_storage::AuroraStorage;
use libc::c_char;
use std::ffi::{CStr, CString};
use std::slice;

lazy_static! {
    static ref STORAGES: HandleStore<AuroraStorage> = HandleStore::new();
}

pub fn create(name: *const c_char, config: *const c_char, credentials: *const c_char) -> ErrorCode {
    ErrorCode::Success
}

pub fn delete(name: *const c_char, config: *const c_char, credentials: *const c_char) -> ErrorCode {
    ErrorCode::Success
}

pub fn open(name: *const c_char, config: *const c_char, runtime_config: *const c_char, credentials: *const c_char, handle_p: *mut i32) -> ErrorCode {
    ErrorCode::Success
}

pub fn close(storage_handle: i32) -> ErrorCode {
    if STORAGES.remove(storage_handle) {
        ErrorCode::Success
    }
    else {
        ErrorCode::InvalidStorageHandle
    }
}

pub fn add_record(storage_handle: i32, type_: *const c_char, id: *const c_char, value: *const u8, value_len: usize, tags_json: *const c_char) -> ErrorCode {
    ErrorCode::Success
}

pub fn get_record(storage_handle: i32, type_: *const c_char, id: *const c_char, options_json: *const c_char, record_handle_p: *mut i32) -> ErrorCode {
    ErrorCode::Success
}

pub fn delete_record(storage_handle: i32, type_: *const c_char, id: *const c_char) -> ErrorCode {
    ErrorCode::Success
}

pub fn add_record_tags(storage_handle: i32, type_: *const c_char, id: *const c_char, tags_json: *const c_char) -> ErrorCode {
    ErrorCode::Success
}

pub fn update_record_value(storage_handle: i32, type_: *const c_char, id: *const c_char, value: *const c_char, value_len: usize) -> ErrorCode {
    ErrorCode::Success
}

pub fn update_record_tags(storage_handle: i32, type_: *const c_char, id: *const c_char, tags_json: *const c_char) -> ErrorCode {
    ErrorCode::Success
}

pub fn delete_record_tags(storage_handle: i32, type_: *const c_char, id: *const c_char, tags_names_json: *const c_char) -> ErrorCode {
    ErrorCode::Success
}

pub fn get_record_type(storage_handle: i32, record_handle: i32, type_p: *mut *const c_char) -> ErrorCode {
    ErrorCode::Success
}

pub fn get_record_id(storage_handle: i32, record_handle: i32, id_p: *mut *const c_char) -> ErrorCode {
    ErrorCode::Success
}

pub fn get_record_value(storage_handle: i32, record_handle: i32, value_p: *mut *const u8, value_len_p: *mut usize) -> ErrorCode {
    ErrorCode::Success
}

pub fn get_record_tags(storage_handle: i32, record_handle: i32, tags_json_p: *mut *const c_char) -> ErrorCode {
    ErrorCode::Success
}

pub fn free_record(storage_handle: i32, record_handle: i32) -> ErrorCode {
    ErrorCode::Success
}

pub fn search_records(storage_handle: i32, type_: *const c_char, query_json: *const c_char, options_json: *const c_char, search_handle_p: *mut i32) -> ErrorCode {
    ErrorCode::Success
}

pub fn search_all_records(storage_handle: i32, search_handle_p: *mut i32) -> ErrorCode {
    ErrorCode::Success
}

pub fn get_search_total_count(storage_handle: i32, search_handle: i32, total_count_p: *mut usize) -> ErrorCode {
    ErrorCode::Success
}

pub fn fetch_search_next_record(storage_handle: i32, search_handle: i32, record_handle_p: *mut i32) -> ErrorCode {
    ErrorCode::Success
}

pub fn free_search(storage_handle: i32, search_handle: i32) -> ErrorCode {
    ErrorCode::Success
}
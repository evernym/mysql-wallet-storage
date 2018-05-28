use utils::handle_store::HandleStore;
use utils::multi_pool::MultiPool;
use errors::error_code::ErrorCode;
use aurora_storage::{AuroraStorage};
use aurora_storage::statement::Statement;
use libc::c_char;
use std::ffi::CStr;
use std::slice;
use serde_json;
use std::collections::HashMap;

// TODO
//  - Move create/open/delete logic from api file into aurora_storage file
//  - Modify tests to use prepare/cleanup
//  - Change the way we are connecting to the DB - OptsBuilder / CLIENT_FOUND_ROWS flag

macro_rules! c_char_to_str {
    ($x: expr) => {
        match unsafe { CStr::from_ptr($x).to_str() } {
            Err(_) => return ErrorCode::InvalidState,
            Ok(s) => s,
        }
    }
}

#[derive(Deserialize)]
struct StorageConfig {
    db_name: String,
    read_host: String,
    write_host: String,
    port: u16,
}

#[derive(Deserialize)]
struct StorageCredentials {
    user: String,
    pass: String,
}

lazy_static! {
    static ref STORAGES: HandleStore<AuroraStorage<'static>> = HandleStore::new();
    static ref CONNECTIONS: MultiPool = MultiPool::new();
}

#[no_mangle]
pub extern "C" fn create_storage(name: *const c_char, config: *const c_char, credentials: *const c_char, metadata: *const c_char) -> ErrorCode {
    let name = c_char_to_str!(name);
    let config: StorageConfig = check_result!(serde_json::from_str(c_char_to_str!(config)), ErrorCode::InvalidStructure);
    let credentials: StorageCredentials = check_result!(serde_json::from_str(c_char_to_str!(credentials)), ErrorCode::InvalidStructure);
    let metadata = c_char_to_str!(metadata);

    let write_connection_string = format!("mysql://{}:{}@{}:{}/{}", credentials.user, credentials.pass, config.write_host, config.port, config.db_name);

    let write_pool = check_option!(CONNECTIONS.get(&write_connection_string), ErrorCode::IOError);

    check_result!(write_pool.prep_exec(Statement::InsertWallet.as_str(), params!{name, metadata}), ErrorCode::IOError);

    ErrorCode::Success
}

#[no_mangle]
pub extern "C" fn delete_storage(name: *const c_char, config: *const c_char, credentials: *const c_char) -> ErrorCode {
    let name = c_char_to_str!(name);
    let config: StorageConfig = check_result!(serde_json::from_str(c_char_to_str!(config)), ErrorCode::InvalidStructure);
    let credentials: StorageCredentials = check_result!(serde_json::from_str(c_char_to_str!(credentials)), ErrorCode::InvalidStructure);

    let write_connection_string = format!("mysql://{}:{}@{}:{}/{}", credentials.user, credentials.pass, config.write_host, config.port, config.db_name);

    let write_pool = check_option!(CONNECTIONS.get(&write_connection_string), ErrorCode::IOError);

    let result = check_result!(write_pool.prep_exec(Statement::DeleteWallet.as_str(), params!{name}), ErrorCode::IOError);

    if result.affected_rows() != 1 {
        return ErrorCode::InvalidState;
    }

    ErrorCode::Success
}

#[no_mangle]
pub extern "C" fn open_storage(name: *const c_char, config: *const c_char, _runtime_config: *const c_char, credentials: *const c_char, handle_p: *mut i32) -> ErrorCode {
    let name = c_char_to_str!(name);
    let config: StorageConfig = check_result!(serde_json::from_str(c_char_to_str!(config)), ErrorCode::InvalidStructure);
    let credentials: StorageCredentials = check_result!(serde_json::from_str(c_char_to_str!(credentials)), ErrorCode::InvalidStructure);

    let read_connection_string = format!("mysql://{}:{}@{}:{}/{}", credentials.user, credentials.pass, config.read_host, config.port, config.db_name);
    let write_connection_string = format!("mysql://{}:{}@{}:{}/{}", credentials.user, credentials.pass, config.write_host, config.port, config.db_name);

    let read_pool = check_option!(CONNECTIONS.get(&read_connection_string), ErrorCode::IOError);
    let write_pool = check_option!(CONNECTIONS.get(&write_connection_string), ErrorCode::IOError);

    let mut result = check_result!(read_pool.prep_exec(Statement::GetWalletID.as_str(), params!{name}), ErrorCode::IOError);
    let wallet_id: u64 = check_option!(check_result!(check_option!(result.next(), ErrorCode::InvalidState), ErrorCode::IOError).get(0), ErrorCode::InvalidState);

    let storage = AuroraStorage::new(wallet_id, read_pool, write_pool);
    let handle = STORAGES.insert(storage);

    unsafe { *handle_p = handle; }

    ErrorCode::Success
}

#[no_mangle]
pub extern "C" fn close_storage(storage_handle: i32) -> ErrorCode {
    if STORAGES.remove(storage_handle) {
        ErrorCode::Success
    }
    else {
        ErrorCode::InvalidState
    }
}

#[no_mangle]
pub extern "C" fn add_record(storage_handle: i32, type_p: *const c_char, id_p: *const c_char, value_p: *const u8, value_len: usize, tags_json_p: *const c_char) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidState);

    let type_ = c_char_to_str!(type_p);
    let id = c_char_to_str!(id_p);
    let tags: HashMap<String, serde_json::Value> = check_result!(serde_json::from_str(c_char_to_str!(tags_json_p)), ErrorCode::InvalidStructure);

    let mut value: Vec<u8> = Vec::new();
    unsafe { value.extend_from_slice(slice::from_raw_parts(value_p, value_len)); }

    storage.add_record(&type_, &id, &value, &tags)
}

#[no_mangle]
pub extern "C" fn get_record(storage_handle: i32, type_p: *const c_char, id_p: *const c_char, options_json_p: *const c_char, record_handle_p: *mut i32) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidState);

    let type_ = c_char_to_str!(type_p);
    let id = c_char_to_str!(id_p);
    let options = c_char_to_str!(options_json_p);

    storage.fetch_record(type_, id, options, record_handle_p)
}

#[no_mangle]
pub extern "C" fn delete_record(storage_handle: i32, type_p: *const c_char, id_p: *const c_char) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidState);

    let type_ = c_char_to_str!(type_p);
    let id = c_char_to_str!(id_p);

    storage.delete_record(&type_, &id)
}

#[no_mangle]
pub extern "C" fn update_record_value(storage_handle: i32, type_p: *const c_char, id_p: *const c_char, value_p: *const u8, value_len: usize) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidState);

    let type_ = c_char_to_str!(type_p);
    let id = c_char_to_str!(id_p);

    let mut value: Vec<u8> = Vec::new();
    unsafe { value.extend_from_slice(slice::from_raw_parts(value_p, value_len)); }

    storage.update_record_value(&type_, &id, &value)
}

#[no_mangle]
pub extern "C" fn add_record_tags(storage_handle: i32, type_p: *const c_char, id_p: *const c_char, tags_json_p: *const c_char) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidState);

    let type_ = c_char_to_str!(type_p);
    let id = c_char_to_str!(id_p);
    let tags: HashMap<String, serde_json::Value> = check_result!(serde_json::from_str(c_char_to_str!(tags_json_p)), ErrorCode::InvalidStructure);

    storage.add_record_tags(&type_, &id, &tags)
}

#[no_mangle]
pub extern "C" fn update_record_tags(storage_handle: i32, type_p: *const c_char, id_p: *const c_char, tags_json_p: *const c_char) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidState);

    let type_ = c_char_to_str!(type_p);
    let id = c_char_to_str!(id_p);
    let tags: HashMap<String, serde_json::Value> = check_result!(serde_json::from_str(c_char_to_str!(tags_json_p)), ErrorCode::InvalidStructure);

    storage.update_record_tags(&type_, &id, &tags)
}

#[no_mangle]
pub extern "C" fn delete_record_tags(storage_handle: i32, type_p: *const c_char, id_p: *const c_char, tag_names_json_p: *const c_char) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidState);

    let type_ = c_char_to_str!(type_p);
    let id = c_char_to_str!(id_p);
    let tag_names: Vec<String> = check_result!(serde_json::from_str(c_char_to_str!(tag_names_json_p)), ErrorCode::InvalidStructure);

    storage.delete_record_tags(&type_, &id, &tag_names)
}

#[no_mangle]
pub extern "C" fn get_record_type(storage_handle: i32, record_handle: i32, type_p: *mut *const c_char) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidState);
    let record = check_option!(storage.get_record(record_handle), ErrorCode::InvalidState);

    match record.type_ {
        None => ErrorCode::InvalidState,
        Some(ref type_) => {
            unsafe { *type_p = type_.as_ptr(); }
            ErrorCode::Success
        }
    }
}

#[no_mangle]
pub extern "C" fn get_record_id(storage_handle: i32, record_handle: i32, id_p: *mut *const c_char) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidState);
    let record = check_option!(storage.get_record(record_handle), ErrorCode::InvalidState);

    unsafe { *id_p = record.id.as_ptr(); }
    ErrorCode::Success
}

#[no_mangle]
pub extern "C" fn get_record_value(storage_handle: i32, record_handle: i32, value_p: *mut *const u8, value_len_p: *mut usize) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidState);
    let record = check_option!(storage.get_record(record_handle), ErrorCode::InvalidState);

    match record.value {
        None => ErrorCode::InvalidState,
        Some(ref value) => {
            unsafe {
                *value_p = value.as_ptr();
                *value_len_p = value.len();
            }
            ErrorCode::Success
        }
    }
}

#[no_mangle]
pub extern "C" fn get_record_tags(storage_handle: i32, record_handle: i32, tags_json_p: *mut *const c_char) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidState);
    let record = check_option!(storage.get_record(record_handle), ErrorCode::InvalidState);

    match record.tags {
        None => ErrorCode::InvalidState,
        Some(ref tags) => {
            unsafe { *tags_json_p = tags.as_ptr(); }
            ErrorCode::Success
        }
    }
}

#[no_mangle]
pub extern "C" fn free_record(storage_handle: i32, record_handle: i32) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidState);
    storage.free_record(record_handle)
}

#[no_mangle]
pub extern "C" fn get_metadata(storage_handle: i32, metadata_ptr: *mut *const c_char, metadata_handle_ptr: *mut i32) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidState);
    let metadata = storage.get_metadata();

    match metadata {
        Err(e) => e,
        Ok((metadata, handle)) => {
            unsafe {
                *metadata_ptr = metadata.as_ptr();
                *metadata_handle_ptr = handle;
            }
            ErrorCode::Success
        }
    }
}

#[no_mangle]
pub extern "C" fn set_metadata(storage_handle: i32, metadata_ptr: *const c_char) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidState);
    let metadata = c_char_to_str!(metadata_ptr);
    storage.set_metadata(metadata)
}

#[no_mangle]
pub extern "C" fn free_metadata(storage_handle: i32, metadata_handle: i32) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidState);
    storage.free_metadata(metadata_handle)
}

#[no_mangle]
pub extern "C" fn search_records(storage_handle: i32, type_p: *const c_char, query_json_p: *const c_char, options_json_p: *const c_char, search_handle_p: *mut i32) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidState);

    let query_json = c_char_to_str!(query_json_p);
    let options_json = c_char_to_str!(options_json_p);
    let type_ = c_char_to_str!(type_p);

    storage.search_records(type_, query_json, options_json, search_handle_p)
}

#[no_mangle]
pub extern "C" fn search_all_records(storage_handle: i32, search_handle_p: *mut i32) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidState);

    storage.search_all_records(search_handle_p)
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn get_search_total_count(storage_handle: i32, search_handle: i32, total_count_p: *mut usize) -> ErrorCode {
    ErrorCode::InvalidState
}

#[no_mangle]
pub extern "C" fn fetch_search_next_record(storage_handle: i32, search_handle: i32, record_handle_p: *mut i32) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidState);
    storage.fetch_search_next_record(search_handle, record_handle_p)
}

#[no_mangle]
pub extern "C" fn free_search(storage_handle: i32, search_handle: i32) -> ErrorCode {
    match STORAGES.get(storage_handle) {
        None => ErrorCode::InvalidState,
        Some(storage) => storage.free_search(search_handle)
    }
}

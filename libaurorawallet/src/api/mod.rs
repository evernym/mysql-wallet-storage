use utils::handle_store::HandleStore;
use utils::multi_pool::MultiPool;
use errors::error_code::ErrorCode;
use aurora_storage::{AuroraStorage};
use libc::c_char;
use std::ffi::CStr;
use std::slice;
use serde_json;
use std::collections::HashMap;

// TODO
//  - Move create/open/delete logic from api file into aurora_storage file
//  - Modify tests to use prepare/cleanup
//  - Search function

macro_rules! c_char_to_str {
    ($x: expr) => {
        match unsafe { CStr::from_ptr($x).to_str() } {
            Err(_) => return ErrorCode::InvalidEncoding,
            Ok(s) => s,
        }
    }
}

#[derive(Deserialize)]
struct StorageConfig {
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
    static ref STORAGES: HandleStore<AuroraStorage> = HandleStore::new();
    static ref CONNECTIONS: MultiPool = MultiPool::new();
}

pub extern "C" fn create(name: *const c_char, config: *const c_char, credentials: *const c_char, metadata: *const c_char) -> ErrorCode {
    let name = c_char_to_str!(name);
    let config: StorageConfig = check_result!(serde_json::from_str(c_char_to_str!(config)), ErrorCode::InvalidJSON);
    let credentials: StorageCredentials = check_result!(serde_json::from_str(c_char_to_str!(credentials)), ErrorCode::InvalidJSON);
    let metadata = c_char_to_str!(metadata);

    let write_connection_string = format!("mysql://{}:{}@{}:{}/wallet", credentials.user, credentials.pass, config.write_host, config.port);

    let write_pool = check_option!(CONNECTIONS.get(&write_connection_string), ErrorCode::ConnectionError);

    check_result!(write_pool.prep_exec(r"INSERT INTO wallets(name, metadata) VALUES (:name, :metadata)", params!{name, metadata}), ErrorCode::DatabaseError);

    ErrorCode::Success
}

pub extern "C" fn delete(name: *const c_char, config: *const c_char, credentials: *const c_char) -> ErrorCode {
    let name = c_char_to_str!(name);
    let config: StorageConfig = check_result!(serde_json::from_str(c_char_to_str!(config)), ErrorCode::InvalidJSON);
    let credentials: StorageCredentials = check_result!(serde_json::from_str(c_char_to_str!(credentials)), ErrorCode::InvalidJSON);

    let write_connection_string = format!("mysql://{}:{}@{}:{}/wallet", credentials.user, credentials.pass, config.write_host, config.port);

    let write_pool = check_option!(CONNECTIONS.get(&write_connection_string), ErrorCode::ConnectionError);

    let result = check_result!(write_pool.prep_exec(r"DELETE FROM wallets WHERE name = :name", params!{name}), ErrorCode::DatabaseError);

    if result.affected_rows() != 1 {
        return ErrorCode::UnknownWalletName;
    }

    ErrorCode::Success
}

pub extern "C" fn open(name: *const c_char, config: *const c_char, _runtime_config: *const c_char, credentials: *const c_char, handle_p: *mut i32) -> ErrorCode {
    let name = c_char_to_str!(name);
    let config: StorageConfig = check_result!(serde_json::from_str(c_char_to_str!(config)), ErrorCode::InvalidJSON);
    let credentials: StorageCredentials = check_result!(serde_json::from_str(c_char_to_str!(credentials)), ErrorCode::InvalidJSON);

    let read_connection_string = format!("mysql://{}:{}@{}:{}/wallet", credentials.user, credentials.pass, config.read_host, config.port);
    let write_connection_string = format!("mysql://{}:{}@{}:{}/wallet", credentials.user, credentials.pass, config.write_host, config.port);

    let read_pool = check_option!(CONNECTIONS.get(&read_connection_string), ErrorCode::ConnectionError);
    let write_pool = check_option!(CONNECTIONS.get(&write_connection_string), ErrorCode::ConnectionError);

    let mut result = check_result!(read_pool.prep_exec(r"SELECT id FROM wallets WHERE name = :name", params!{name}), ErrorCode::DatabaseError);
    let wallet_id: u64 = check_option!(check_result!(check_option!(result.next(), ErrorCode::UnknownWalletName), ErrorCode::DatabaseError).get(0), ErrorCode::UnknownWalletName);

    let storage = AuroraStorage::new(wallet_id, read_pool, write_pool);
    let handle = STORAGES.insert(storage);

    unsafe { *handle_p = handle; }

    ErrorCode::Success
}

pub extern "C" fn close(storage_handle: i32) -> ErrorCode {
    if STORAGES.remove(storage_handle) {
        ErrorCode::Success
    }
    else {
        ErrorCode::InvalidStorageHandle
    }
}

pub extern "C" fn add_record(storage_handle: i32, type_p: *const c_char, id_p: *const c_char, value_p: *const u8, value_len: usize, tags_json_p: *const c_char) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidStorageHandle);

    let type_ = c_char_to_str!(type_p);
    let id = c_char_to_str!(id_p);
    let tags: HashMap<String, serde_json::Value> = check_result!(serde_json::from_str(c_char_to_str!(tags_json_p)), ErrorCode::InvalidJSON);

    let mut value: Vec<u8> = Vec::new();
    unsafe { value.extend_from_slice(slice::from_raw_parts(value_p, value_len)); }

    storage.add_record(&type_, &id, &value, &tags)
}

pub extern "C" fn get_record(storage_handle: i32, type_p: *const c_char, id_p: *const c_char, options_json_p: *const c_char, record_handle_p: *mut i32) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidStorageHandle);

    let type_ = c_char_to_str!(type_p);
    let id = c_char_to_str!(id_p);
    let options = c_char_to_str!(options_json_p);

    storage.fetch_record(type_, id, options, record_handle_p)
}

pub extern "C" fn delete_record(storage_handle: i32, type_p: *const c_char, id_p: *const c_char) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidStorageHandle);

    let type_ = c_char_to_str!(type_p);
    let id = c_char_to_str!(id_p);

    storage.delete_record(&type_, &id)
}

pub extern "C" fn update_record_value(storage_handle: i32, type_p: *const c_char, id_p: *const c_char, value_p: *const u8, value_len: usize) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidStorageHandle);

    let type_ = c_char_to_str!(type_p);
    let id = c_char_to_str!(id_p);

    let mut value: Vec<u8> = Vec::new();
    unsafe { value.extend_from_slice(slice::from_raw_parts(value_p, value_len)); }

    storage.update_record_value(&type_, &id, &value)
}

pub extern "C" fn add_record_tags(storage_handle: i32, type_p: *const c_char, id_p: *const c_char, tags_json_p: *const c_char) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidStorageHandle);

    let type_ = c_char_to_str!(type_p);
    let id = c_char_to_str!(id_p);
    let tags: HashMap<String, serde_json::Value> = check_result!(serde_json::from_str(c_char_to_str!(tags_json_p)), ErrorCode::InvalidJSON);

    storage.add_record_tags(&type_, &id, &tags)
}

pub extern "C" fn update_record_tags(storage_handle: i32, type_p: *const c_char, id_p: *const c_char, tags_json_p: *const c_char) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidStorageHandle);

    let type_ = c_char_to_str!(type_p);
    let id = c_char_to_str!(id_p);
    let tags: HashMap<String, serde_json::Value> = check_result!(serde_json::from_str(c_char_to_str!(tags_json_p)), ErrorCode::InvalidJSON);

    storage.update_record_tags(&type_, &id, &tags)
}

pub extern "C" fn delete_record_tags(storage_handle: i32, type_p: *const c_char, id_p: *const c_char, tag_names_json_p: *const c_char) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidStorageHandle);

    let type_ = c_char_to_str!(type_p);
    let id = c_char_to_str!(id_p);
    let tag_names: Vec<String> = check_result!(serde_json::from_str(c_char_to_str!(tag_names_json_p)), ErrorCode::InvalidJSON);

    storage.delete_record_tags(&type_, &id, &tag_names)
}

pub extern "C" fn get_record_type(storage_handle: i32, record_handle: i32, type_p: *mut *const c_char) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidStorageHandle);
    let record = check_option!(storage.get_record(record_handle), ErrorCode::InvalidRecordHandle);

    match record.type_ {
        None => ErrorCode::TypeNotFetched,
        Some(ref type_) => {
            unsafe { *type_p = type_.as_ptr(); }
            ErrorCode::Success
        }
    }
}

pub extern "C" fn get_record_id(storage_handle: i32, record_handle: i32, id_p: *mut *const c_char) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidStorageHandle);
    let record = check_option!(storage.get_record(record_handle), ErrorCode::InvalidRecordHandle);

    unsafe { *id_p = record.id.as_ptr(); }
    ErrorCode::Success
}

pub extern "C" fn get_record_value(storage_handle: i32, record_handle: i32, value_p: *mut *const u8, value_len_p: *mut usize) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidStorageHandle);
    let record = check_option!(storage.get_record(record_handle), ErrorCode::InvalidRecordHandle);

    match record.value {
        None => ErrorCode::ValueNotFetched,
        Some(ref value) => {
            unsafe {
                *value_p = value.as_ptr();
                *value_len_p = value.len();
            }
            ErrorCode::Success
        }
    }
}

pub extern "C" fn get_record_tags(storage_handle: i32, record_handle: i32, tags_json_p: *mut *const c_char) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidStorageHandle);
    let record = check_option!(storage.get_record(record_handle), ErrorCode::InvalidRecordHandle);

    match record.tags {
        None => ErrorCode::TagsNotFetched,
        Some(ref tags) => {
            unsafe { *tags_json_p = tags.as_ptr(); }
            ErrorCode::Success
        }
    }
}

pub extern "C" fn free_record(storage_handle: i32, record_handle: i32) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidStorageHandle);
    storage.free_record(record_handle)
}

pub extern "C" fn get_metadata(storage_handle: i32, metadata_ptr: *mut *const c_char, metadata_handle_ptr: *mut i32) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidStorageHandle);
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

pub extern "C" fn set_metadata(storage_handle: i32, metadata_ptr: *const c_char) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidStorageHandle);
    let metadata = c_char_to_str!(metadata_ptr);
    storage.set_metadata(metadata)
}

pub extern "C" fn free_metadata(storage_handle: i32, metadata_handle: i32) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidStorageHandle);
    storage.free_metadata(metadata_handle)
}

#[allow(unused_variables)]
pub extern "C" fn search_records(storage_handle: i32, type_: *const c_char, query_json: *const c_char, options_json: *const c_char, search_handle_p: *mut i32) -> ErrorCode {
    ErrorCode::Success
}

#[allow(unused_variables)]
pub extern "C" fn search_all_records(storage_handle: i32, search_handle_p: *mut i32) -> ErrorCode {
    ErrorCode::Success
}

#[allow(unused_variables)]
pub extern "C" fn get_search_total_count(storage_handle: i32, search_handle: i32, total_count_p: *mut usize) -> ErrorCode {
    ErrorCode::Success
}

#[allow(unused_variables)]
pub extern "C" fn fetch_search_next_record(storage_handle: i32, search_handle: i32, record_handle_p: *mut i32) -> ErrorCode {
    ErrorCode::Success
}

pub extern "C" fn free_search(storage_handle: i32, search_handle: i32) -> ErrorCode {
    match STORAGES.get(storage_handle) {
        None => ErrorCode::InvalidStorageHandle,
        Some(storage) => storage.free_search(search_handle)
    }
}

use utils::handle_store::HandleStore;
use utils::multi_pool::MultiPool;
use utils::error_code::ErrorCode;
use aurora_storage::{AuroraStorage, FetchOptions, Record};
use libc::c_char;
use std::ffi::{CStr, CString};
use std::slice;
use serde_json;
use mysql::Pool;
use std::sync::Arc;
use std::collections::HashMap;

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

pub fn create(name: *const c_char, config: *const c_char, credentials: *const c_char) -> ErrorCode {
    let name = c_char_to_str!(name);
    let config: StorageConfig = check_result!(serde_json::from_str(c_char_to_str!(config)), ErrorCode::InvalidJSON);
    let credentials: StorageCredentials = check_result!(serde_json::from_str(c_char_to_str!(credentials)), ErrorCode::InvalidJSON);

    let write_connection_string = format!("mysql://{}:{}@{}:{}/wallet", credentials.user, credentials.pass, config.write_host, config.port);

    let write_pool = check_option!(CONNECTIONS.get(&write_connection_string), ErrorCode::ConnectionError);

    check_result!(write_pool.prep_exec(r"INSERT INTO wallets(name) VALUES (:name)", params!{name}), ErrorCode::DatabaseError);

    ErrorCode::Success
}

pub fn delete(name: *const c_char, config: *const c_char, credentials: *const c_char) -> ErrorCode {
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

pub fn open(name: *const c_char, config: *const c_char, _runtime_config: *const c_char, credentials: *const c_char, handle_p: *mut i32) -> ErrorCode {
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

pub fn close(storage_handle: i32) -> ErrorCode {
    if STORAGES.remove(storage_handle) {
        ErrorCode::Success
    }
    else {
        ErrorCode::InvalidStorageHandle
    }
}

pub fn add_record(storage_handle: i32, type_p: *const c_char, id_p: *const c_char, value_p: *const u8, value_len: usize, tags_json_p: *const c_char) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidStorageHandle);

    let type_ = c_char_to_str!(type_p);
    let id = c_char_to_str!(id_p);
    let tags: HashMap<String, String> = check_result!(serde_json::from_str(c_char_to_str!(tags_json_p)), ErrorCode::InvalidJSON);

    let mut value: Vec<u8> = Vec::new();
    unsafe { value.extend_from_slice(slice::from_raw_parts(value_p, value_len)); }

    storage.add_record(&type_, &id, &value, &tags)
}

pub fn get_record(storage_handle: i32, type_p: *const c_char, id_p: *const c_char, options_json_p: *const c_char, record_handle_p: *mut i32) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidStorageHandle);

    let type_ = c_char_to_str!(type_p);
    let id = c_char_to_str!(id_p);
    let options = c_char_to_str!(options_json_p);

    storage.fetch_record(type_, id, options, record_handle_p)
}

pub fn delete_record(storage_handle: i32, type_p: *const c_char, id_p: *const c_char) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidStorageHandle);

    let type_ = c_char_to_str!(type_p);
    let id = c_char_to_str!(id_p);

    storage.delete_record(&type_, &id)
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

pub fn get_record_id(storage_handle: i32, record_handle: i32, id_p: *mut *const c_char) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidStorageHandle);
    let record = check_option!(storage.get_record(record_handle), ErrorCode::InvalidRecordHandle);

    unsafe { *id_p = record.id.as_ptr(); }
    ErrorCode::Success
}

pub fn get_record_value(storage_handle: i32, record_handle: i32, value_p: *mut *const u8, value_len_p: *mut usize) -> ErrorCode {
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

pub fn get_record_tags(storage_handle: i32, record_handle: i32, tags_json_p: *mut *const c_char) -> ErrorCode {
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

pub fn free_record(storage_handle: i32, record_handle: i32) -> ErrorCode {
    match STORAGES.get(storage_handle) {
        None => ErrorCode::InvalidStorageHandle,
        Some(storage) => storage.free_record(record_handle)
    }
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
    match STORAGES.get(storage_handle) {
        None => ErrorCode::InvalidStorageHandle,
        Some(storage) => storage.free_search(search_handle)
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use super::*;
    use std::ptr;
    use self::rand::{thread_rng, Rng};

    fn random_name() -> String {
        thread_rng().gen_ascii_chars().take(10).collect()
    }

    fn open_storage() -> i32 {
        let name = CString::new("test-wallet").unwrap();
        let config = CString::new(r##"{"read_host":"localhost", "write_host":"localhost", "port": 3306}"##).unwrap();
        let runtime_config = CString::new("").unwrap();
        let credentials = CString::new(r##"{"user": "wallet", "pass": "wallet"}"##).unwrap();
        let mut handle: i32 = -1;

        let err = open(name.as_ptr(), config.as_ptr(), runtime_config.as_ptr(), credentials.as_ptr(), &mut handle);

        assert_eq!(err, ErrorCode::Success);
        handle
    }

    #[test]
    fn test_open() {
        let h = open_storage();
    }

    #[test]
    fn test_create_and_delete() {
        let name = CString::new(random_name()).unwrap();
        let config = CString::new(r##"{"read_host":"localhost", "write_host":"localhost", "port": 3306}"##).unwrap();
        let runtime_config = CString::new("").unwrap();
        let credentials = CString::new(r##"{"user": "wallet", "pass": "wallet"}"##).unwrap();

        let err = create(name.as_ptr(), config.as_ptr(), credentials.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let mut handle: i32 = -1;
        let err = open(name.as_ptr(), config.as_ptr(), runtime_config.as_ptr(), credentials.as_ptr(), &mut handle);
        assert_eq!(err, ErrorCode::Success);

        let err = close(handle);
        assert_eq!(err, ErrorCode::Success);

        let err = delete(name.as_ptr(), config.as_ptr(), credentials.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_insert_item_with_tags_then_fetch_all() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];
        let tags_json = CString::new(r##"{"tag1": "value1", "tag2": "value2", "~tag3": "value3"}"##).unwrap();
        let options_json = CString::new(r##"{"fetch_value": true, "fetch_tags": true}"##).unwrap();
        let mut record_handle = -1;
        let mut id_p: *const c_char = ptr::null_mut();
        let mut value_p: *const u8 = ptr::null_mut();
        let mut value_len_p = 0;
        let mut tags_json_p: *const c_char = ptr::null_mut();

        let err = add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = get_record_id(handle, record_handle, &mut id_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe{CStr::from_ptr(id_p)}.to_owned(), id);

        let err = get_record_value(handle, record_handle, &mut value_p, &mut value_len_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe{slice::from_raw_parts(value_p, value_len_p)}, value.as_slice());

        let err = get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::Success);

        let expected_tags_map: HashMap<String, String> = serde_json::from_slice(tags_json.as_bytes()).unwrap();
        let tags_map: HashMap<String, String> = serde_json::from_str(unsafe{CStr::from_ptr(tags_json_p)}.to_str().unwrap()).unwrap();
        assert_eq!(tags_map, expected_tags_map);

        let err = delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_insert_item_with_tags_then_fetch_only_value() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];
        let tags_json = CString::new(r##"{"tag1": "value1", "tag2": "value2", "~tag3": "value3"}"##).unwrap();
        let options_json = CString::new(r##"{"fetch_value": true, "fetch_tags": false}"##).unwrap();
        let mut record_handle = -1;
        let mut id_p: *const c_char = ptr::null_mut();
        let mut value_p: *const u8 = ptr::null_mut();
        let mut value_len_p = 0;
        let mut tags_json_p: *const c_char = ptr::null_mut();

        let err = add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = get_record_id(handle, record_handle, &mut id_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe{CStr::from_ptr(id_p)}.to_owned(), id);

        let err = get_record_value(handle, record_handle, &mut value_p, &mut value_len_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe{slice::from_raw_parts(value_p, value_len_p)}, value.as_slice());

        let err = get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::TagsNotFetched);

        let err = delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_insert_item_with_tags_then_fetch_only_tags() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];
        let tags_json = CString::new(r##"{"tag1": "value1", "tag2": "value2", "~tag3": "value3"}"##).unwrap();
        let options_json = CString::new(r##"{"fetch_value": false, "fetch_tags": true}"##).unwrap();
        let mut record_handle = -1;
        let mut id_p: *const c_char = ptr::null_mut();
        let mut value_p: *const u8 = ptr::null_mut();
        let mut value_len_p = 0;
        let mut tags_json_p: *const c_char = ptr::null_mut();

        let err = add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = get_record_id(handle, record_handle, &mut id_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe{CStr::from_ptr(id_p)}.to_owned(), id);

        let err = get_record_value(handle, record_handle, &mut value_p, &mut value_len_p);
        assert_eq!(err, ErrorCode::ValueNotFetched);

        let err = get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::Success);

        let expected_tags_map: HashMap<String, String> = serde_json::from_slice(tags_json.as_bytes()).unwrap();
        let tags_map: HashMap<String, String> = serde_json::from_str(unsafe{CStr::from_ptr(tags_json_p)}.to_str().unwrap()).unwrap();
        assert_eq!(tags_map, expected_tags_map);

        let err = delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_insert_item_with_tags_then_fetch_none() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];
        let tags_json = CString::new(r##"{"tag1": "value1", "tag2": "value2", "~tag3": "value3"}"##).unwrap();
        let options_json = CString::new(r##"{"fetch_value": false, "fetch_tags": false}"##).unwrap();
        let mut record_handle = -1;
        let mut id_p: *const c_char = ptr::null_mut();
        let mut value_p: *const u8 = ptr::null_mut();
        let mut value_len_p = 0;
        let mut tags_json_p: *const c_char = ptr::null_mut();

        let err = add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = get_record_id(handle, record_handle, &mut id_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe{CStr::from_ptr(id_p)}.to_owned(), id);

        let err = get_record_value(handle, record_handle, &mut value_p, &mut value_len_p);
        assert_eq!(err, ErrorCode::ValueNotFetched);

        let err = get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::TagsNotFetched);

        let err = delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_insert_item_with_tags_then_fetch_default() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];
        let tags_json = CString::new(r##"{"tag1": "value1", "tag2": "value2", "~tag3": "value3"}"##).unwrap();
        let options_json = CString::new(r##"{}"##).unwrap();
        let mut record_handle = -1;
        let mut id_p: *const c_char = ptr::null_mut();
        let mut value_p: *const u8 = ptr::null_mut();
        let mut value_len_p = 0;
        let mut tags_json_p: *const c_char = ptr::null_mut();

        let err = add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = get_record_id(handle, record_handle, &mut id_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe{CStr::from_ptr(id_p)}.to_owned(), id);

        let err = get_record_value(handle, record_handle, &mut value_p, &mut value_len_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe{slice::from_raw_parts(value_p, value_len_p)}, value.as_slice());

        let err = get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::Success);

        let expected_tags_map: HashMap<String, String> = serde_json::from_slice(tags_json.as_bytes()).unwrap();
        let tags_map: HashMap<String, String> = serde_json::from_str(unsafe{CStr::from_ptr(tags_json_p)}.to_str().unwrap()).unwrap();
        assert_eq!(tags_map, expected_tags_map);

        let err = delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_insert_item_without_tags_then_fetch_all() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];
        let tags_json = CString::new(r##"{}"##).unwrap();
        let options_json = CString::new(r##"{"fetch_value": true, "fetch_tags": true}"##).unwrap();
        let mut record_handle = -1;
        let mut id_p: *const c_char = ptr::null_mut();
        let mut value_p: *const u8 = ptr::null_mut();
        let mut value_len_p = 0;
        let mut tags_json_p: *const c_char = ptr::null_mut();

        let err = add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = get_record_id(handle, record_handle, &mut id_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe{CStr::from_ptr(id_p)}.to_owned(), id);

        let err = get_record_value(handle, record_handle, &mut value_p, &mut value_len_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe{slice::from_raw_parts(value_p, value_len_p)}, value.as_slice());

        let err = get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::Success);

        let expected_tags_map: HashMap<String, String> = serde_json::from_slice(tags_json.as_bytes()).unwrap();
        let tags_map: HashMap<String, String> = serde_json::from_str(unsafe{CStr::from_ptr(tags_json_p)}.to_str().unwrap()).unwrap();
        assert_eq!(tags_map, expected_tags_map);

        let err = delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_insert_item_without_tags_then_fetch_only_value() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];
        let tags_json = CString::new(r##"{}"##).unwrap();
        let options_json = CString::new(r##"{"fetch_value": true, "fetch_tags": false}"##).unwrap();
        let mut record_handle = -1;
        let mut id_p: *const c_char = ptr::null_mut();
        let mut value_p: *const u8 = ptr::null_mut();
        let mut value_len_p = 0;
        let mut tags_json_p: *const c_char = ptr::null_mut();

        let err = add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = get_record_id(handle, record_handle, &mut id_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe{CStr::from_ptr(id_p)}.to_owned(), id);

        let err = get_record_value(handle, record_handle, &mut value_p, &mut value_len_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe{slice::from_raw_parts(value_p, value_len_p)}, value.as_slice());

        let err = get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::TagsNotFetched);

        let err = delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_insert_item_without_tags_then_fetch_only_tags() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];
        let tags_json = CString::new(r##"{}"##).unwrap();
        let options_json = CString::new(r##"{"fetch_value": false, "fetch_tags": true}"##).unwrap();
        let mut record_handle = -1;
        let mut id_p: *const c_char = ptr::null_mut();
        let mut value_p: *const u8 = ptr::null_mut();
        let mut value_len_p = 0;
        let mut tags_json_p: *const c_char = ptr::null_mut();

        let err = add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = get_record_id(handle, record_handle, &mut id_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe{CStr::from_ptr(id_p)}.to_owned(), id);

        let err = get_record_value(handle, record_handle, &mut value_p, &mut value_len_p);
        assert_eq!(err, ErrorCode::ValueNotFetched);

        let err = get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::Success);

        let expected_tags_map: HashMap<String, String> = serde_json::from_slice(tags_json.as_bytes()).unwrap();
        let tags_map: HashMap<String, String> = serde_json::from_str(unsafe{CStr::from_ptr(tags_json_p)}.to_str().unwrap()).unwrap();
        assert_eq!(tags_map, expected_tags_map);

        let err = delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_insert_item_without_tags_then_fetch_none() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];
        let tags_json = CString::new(r##"{}"##).unwrap();
        let options_json = CString::new(r##"{"fetch_value": false, "fetch_tags": false}"##).unwrap();
        let mut record_handle = -1;
        let mut id_p: *const c_char = ptr::null_mut();
        let mut value_p: *const u8 = ptr::null_mut();
        let mut value_len_p = 0;
        let mut tags_json_p: *const c_char = ptr::null_mut();

        let err = add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = get_record_id(handle, record_handle, &mut id_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe{CStr::from_ptr(id_p)}.to_owned(), id);

        let err = get_record_value(handle, record_handle, &mut value_p, &mut value_len_p);
        assert_eq!(err, ErrorCode::ValueNotFetched);

        let err = get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::TagsNotFetched);

        let err = delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_insert_item_without_tags_then_fetch_default() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];
        let tags_json = CString::new(r##"{}"##).unwrap();
        let options_json = CString::new(r##"{}"##).unwrap();
        let mut record_handle = -1;
        let mut id_p: *const c_char = ptr::null_mut();
        let mut value_p: *const u8 = ptr::null_mut();
        let mut value_len_p = 0;
        let mut tags_json_p: *const c_char = ptr::null_mut();

        let err = add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = get_record_id(handle, record_handle, &mut id_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe{CStr::from_ptr(id_p)}.to_owned(), id);

        let err = get_record_value(handle, record_handle, &mut value_p, &mut value_len_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe{slice::from_raw_parts(value_p, value_len_p)}, value.as_slice());

        let err = get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::Success);

        let expected_tags_map: HashMap<String, String> = serde_json::from_slice(tags_json.as_bytes()).unwrap();
        let tags_map: HashMap<String, String> = serde_json::from_str(unsafe{CStr::from_ptr(tags_json_p)}.to_str().unwrap()).unwrap();
        assert_eq!(tags_map, expected_tags_map);

        let err = delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

        #[test]
    fn test_duplicate_insert_item_without_tags_then_fetch_default() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];
        let tags_json = CString::new(r##"{}"##).unwrap();
        let options_json = CString::new(r##"{}"##).unwrap();
        let mut record_handle = -1;
        let mut id_p: *const c_char = ptr::null_mut();
        let mut value_p: *const u8 = ptr::null_mut();
        let mut value_len_p = 0;
        let mut tags_json_p: *const c_char = ptr::null_mut();

        let err = add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = get_record_id(handle, record_handle, &mut id_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe{CStr::from_ptr(id_p)}.to_owned(), id);

        let err = get_record_value(handle, record_handle, &mut value_p, &mut value_len_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe{slice::from_raw_parts(value_p, value_len_p)}, value.as_slice());

        let err = get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::Success);

        let expected_tags_map: HashMap<String, String> = serde_json::from_slice(tags_json.as_bytes()).unwrap();
        let tags_map: HashMap<String, String> = serde_json::from_str(unsafe{CStr::from_ptr(tags_json_p)}.to_str().unwrap()).unwrap();
        assert_eq!(tags_map, expected_tags_map);

        let err = add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::RecordAlreadExists);

        let err = delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_delete_unknown() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();

        let err = delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::UnknownItem);
    }
}
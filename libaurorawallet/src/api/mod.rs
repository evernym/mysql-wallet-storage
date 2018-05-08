use utils::handle_store::HandleStore;
use utils::multi_pool::MultiPool;
use errors::error_code::ErrorCode;
use aurora_storage::{AuroraStorage};
use libc::c_char;
use std::ffi::{CStr, CString};
use std::slice;
use serde_json;
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
    let tags: HashMap<String, String> = check_result!(serde_json::from_str(c_char_to_str!(tags_json_p)), ErrorCode::InvalidJSON);

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
    let tags: HashMap<String, String> = check_result!(serde_json::from_str(c_char_to_str!(tags_json_p)), ErrorCode::InvalidJSON);

    storage.add_record_tags(&type_, &id, &tags)
}

pub extern "C" fn update_record_tags(storage_handle: i32, type_p: *const c_char, id_p: *const c_char, tags_json_p: *const c_char) -> ErrorCode {
    let storage = check_option!(STORAGES.get(storage_handle), ErrorCode::InvalidStorageHandle);

    let type_ = c_char_to_str!(type_p);
    let id = c_char_to_str!(id_p);
    let tags: HashMap<String, String> = check_result!(serde_json::from_str(c_char_to_str!(tags_json_p)), ErrorCode::InvalidJSON);

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
    match STORAGES.get(storage_handle) {
        None => ErrorCode::InvalidStorageHandle,
        Some(storage) => storage.free_record(record_handle)
    }
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

#[cfg(test)]
mod tests {
    extern crate rand;

    use super::*;
    use std::ptr;
    use self::rand::{thread_rng, Rng};

    /** Helper Functions */

    fn random_string(char_num: usize) -> String {
        thread_rng().gen_ascii_chars().take(char_num).collect()
    }

    fn random_name() -> String {
        random_string(10)
    }

    fn open_storage() -> i32 {
        let name = CString::new("test-wallet").unwrap();
        let config = CString::new(r##"{"read_host":"localhost", "write_host":"localhost", "port": 3306}"##).unwrap();
        let runtime_config = CString::new("").unwrap();
        let credentials = CString::new(r##"{"user": "root", "pass": "Gs)Sj00uuSK;"}"##).unwrap();
        let mut handle: i32 = -1;

        let err = open(name.as_ptr(), config.as_ptr(), runtime_config.as_ptr(), credentials.as_ptr(), &mut handle);

        assert_eq!(err, ErrorCode::Success);
        handle
    }

    /** Storage OPEN Tests */

    #[test]
    fn test_open() {
        open_storage();
    }

    /** Storage CREATE and DELETE Tests */

    #[test]
    fn test_create_and_delete() {
        let name = CString::new(random_name()).unwrap();
        let config = CString::new(r##"{"read_host":"localhost", "write_host":"localhost", "port": 3306}"##).unwrap();
        let runtime_config = CString::new("").unwrap();
        let credentials = CString::new(r##"{"user": "root", "pass": "Gs)Sj00uuSK;"}"##).unwrap();

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

    /** Storage ADD_RECORD, GET_RECORD, DELETE_RECORD Tests */

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
        assert_eq!(err, ErrorCode::UnknownRecord);
    }

    /** Storage UPDATE_RECORD_VALUE Tests */

    #[test]
    fn test_update_record_value() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let tags_json = CString::new(r##"{"tag1": "value1", "tag2": "value2", "~tag3": "value3"}"##).unwrap();

        let initial_value = vec![1, 2, 3, 4];
        let err = add_record(handle, type_.as_ptr(), id.as_ptr(), initial_value.as_ptr(), initial_value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let new_value = vec![2, 5, 8, 13];
        let err = update_record_value(handle, type_.as_ptr(), id.as_ptr(), new_value.as_ptr(), new_value.len());
        assert_eq!(err, ErrorCode::Success);

        let mut record_handle = -1;
        let options_json = CString::new(r##"{"fetch_value": true, "fetch_tags": false}"##).unwrap();
        let err = get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let mut value_p: *const u8 = ptr::null_mut();
        let mut value_len_p = 0;
        let err = get_record_value(handle, record_handle, &mut value_p, &mut value_len_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe{slice::from_raw_parts(value_p, value_len_p)}, new_value.as_slice());

        let err = delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_update_record_value_with_same_value() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let tags_json = CString::new(r##"{"tag1": "value1", "tag2": "value2", "~tag3": "value3"}"##).unwrap();

        let initial_value = vec![1, 2, 3, 4];
        let err = add_record(handle, type_.as_ptr(), id.as_ptr(), initial_value.as_ptr(), initial_value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let new_value = vec![1, 2, 3, 4];
        let err = update_record_value(handle, type_.as_ptr(), id.as_ptr(), new_value.as_ptr(), new_value.len());
        assert_eq!(err, ErrorCode::Success);

        let mut record_handle = -1;
        let options_json = CString::new(r##"{"fetch_value": true, "fetch_tags": false}"##).unwrap();
        let err = get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let mut value_p: *const u8 = ptr::null_mut();
        let mut value_len_p = 0;
        let err = get_record_value(handle, record_handle, &mut value_p, &mut value_len_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe{slice::from_raw_parts(value_p, value_len_p)}, new_value.as_slice());

        let err = delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_update_value_for_invalid_handle() {
        let handle = -1;

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let new_value = vec![1, 2, 3, 4];

        let err = update_record_value(handle, type_.as_ptr(), id.as_ptr(), new_value.as_ptr(), new_value.len());
        assert_eq!(err, ErrorCode::InvalidStorageHandle);
    }

    #[test]
    fn test_update_value_for_unknown_record() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let new_value = vec![1, 2, 3, 4];

        let err = update_record_value(handle, type_.as_ptr(), id.as_ptr(), new_value.as_ptr(), new_value.len());
        assert_eq!(err, ErrorCode::UnknownRecord);
    }

    /** Storage ADD_RECORD_TAGS Tests */

    #[test]
    fn test_add_record_tags() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];

        let tags_json_empty = CString::new(r##"{}"##).unwrap();
        let err = add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json_empty.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let tags_json = CString::new(r##"{"tag1": "value1", "tag2": "value2", "~tag3": "value3"}"##).unwrap();
        let err = add_record_tags(handle, type_.as_ptr(), id.as_ptr(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let mut record_handle = -1;
        let options_json = CString::new(r##"{"fetch_value": true, "fetch_tags": true}"##).unwrap();
        let err = get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let mut tags_json_p: *const c_char = ptr::null_mut();
        let err = get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::Success);

        let expected_tags_map: HashMap<String, String> = serde_json::from_slice(tags_json.as_bytes()).unwrap();
        let tags_map: HashMap<String, String> = serde_json::from_str(unsafe{CStr::from_ptr(tags_json_p)}.to_str().unwrap()).unwrap();
        assert_eq!(tags_map, expected_tags_map);

        let err = delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_add_duplicate_record_tags() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];


        let tags_json_empty = CString::new(r##"{}"##).unwrap();
        let err = add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json_empty.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let tags_json = CString::new(r##"{"tag1": "value1", "tag2": "value2", "~tag3": "value3"}"##).unwrap();
        let err = add_record_tags(handle, type_.as_ptr(), id.as_ptr(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = add_record_tags(handle, type_.as_ptr(), id.as_ptr(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::TagAlreadyExists);

        let mut record_handle = -1;
        let options_json = CString::new(r##"{"fetch_value": true, "fetch_tags": true}"##).unwrap();
        let err = get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let mut tags_json_p: *const c_char = ptr::null_mut();
        let err = get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::Success);

        let expected_tags_map: HashMap<String, String> = serde_json::from_slice(tags_json.as_bytes()).unwrap();
        let tags_map: HashMap<String, String> = serde_json::from_str(unsafe{CStr::from_ptr(tags_json_p)}.to_str().unwrap()).unwrap();
        assert_eq!(tags_map, expected_tags_map);

        let err = delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_add_encrypted_tags_tag_name_too_long() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];

        let tags_json_empty = CString::new(r##"{}"##).unwrap();
        let err = add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json_empty.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let long_tag_name = random_string(257);
        let tags_json = json!({long_tag_name: "tag_value"}).to_string();
        let tags_json = CString::new(tags_json).unwrap();

        let err = add_record_tags(handle, type_.as_ptr(), id.as_ptr(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::TagDataTooLong);

        let err = delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_add_plaintext_tags_tag_name_too_long() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];

        let tags_json_empty = CString::new(r##"{}"##).unwrap();
        let err = add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json_empty.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let mut long_tag_name = random_string(257);
        long_tag_name.push('~');
        let tags_json = json!({long_tag_name: "tag_value"}).to_string();
        let tags_json = CString::new(tags_json).unwrap();

        let err = add_record_tags(handle, type_.as_ptr(), id.as_ptr(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::TagDataTooLong);

        let err = delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_add_encrypted_tags_tag_value_too_long() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];

        let tags_json_empty = CString::new(r##"{}"##).unwrap();
        let err = add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json_empty.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let long_tag_value = random_string(2817);
        let tags_json = json!({"tag_name": long_tag_value}).to_string();
        let tags_json = CString::new(tags_json).unwrap();

        let err = add_record_tags(handle, type_.as_ptr(), id.as_ptr(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::TagDataTooLong);

        let err = delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_add_plaintext_tags_tag_value_too_long() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];

        let tags_json_empty = CString::new(r##"{}"##).unwrap();
        let err = add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json_empty.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let long_tag_value = random_string(2817);
        let tags_json = json!({"~tag_name": long_tag_value}).to_string();
        let tags_json = CString::new(tags_json).unwrap();

        let err = add_record_tags(handle, type_.as_ptr(), id.as_ptr(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::TagDataTooLong);

        let err = delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_add_tags_for_invalid_handle() {
        let handle = -1;

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let tags_json = CString::new(r##"{"tag1": "value1", "tag2": "value2", "~tag3": "value3"}"##).unwrap();

        let err = add_record_tags(handle, type_.as_ptr(), id.as_ptr(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::InvalidStorageHandle);
    }

    #[test]
    fn test_add_tags_for_unknown_record() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let tags_json = CString::new(r##"{"tag1": "value1", "tag2": "value2", "~tag3": "value3"}"##).unwrap();

        let err = add_record_tags(handle, type_.as_ptr(), id.as_ptr(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::UnknownRecord);
    }

    /** Storage UPDATE_RECORD_TAGS Tests */

    #[test]
    fn test_update_record_tags() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];
        let tags_json = CString::new(r##"{"tag1": "value1", "tag2": "value2", "~tag3": "value3"}"##).unwrap();

        let err = add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let new_tags_json = CString::new(r##"{"tag1": "value1_new", "tag2": "value2_new", "~tag3": "value3_new"}"##).unwrap();
        let err = update_record_tags(handle, type_.as_ptr(), id.as_ptr(), new_tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let mut record_handle = -1;
        let options_json = CString::new(r##"{"fetch_value": true, "fetch_tags": true}"##).unwrap();
        let err = get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let mut tags_json_p: *const c_char = ptr::null_mut();
        let err = get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::Success);

        let expected_tags_map: HashMap<String, String> = serde_json::from_slice(new_tags_json.as_bytes()).unwrap();
        let tags_map: HashMap<String, String> = serde_json::from_str(unsafe{CStr::from_ptr(tags_json_p)}.to_str().unwrap()).unwrap();
        assert_eq!(tags_map, expected_tags_map);

        let err = delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_update_record_encrypted_tags_with_same_values() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];
        let tags_json = CString::new(r##"{"tag1": "value1", "tag2": "value2"}"##).unwrap();

        let err = add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let new_tags_json = CString::new(r##"{"tag1": "value1", "tag2": "value2"}"##).unwrap();
        let err = update_record_tags(handle, type_.as_ptr(), id.as_ptr(), new_tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_update_record_unencrypted_tags_with_same_values() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];
        let tags_json = CString::new(r##"{"~tag1": "value1", "~tag2": "value2"}"##).unwrap();

        let err = add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let new_tags_json = CString::new(r##"{"~tag1": "value1", "~tag2": "value2"}"##).unwrap();
        let err = update_record_tags(handle, type_.as_ptr(), id.as_ptr(), new_tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_update_unknown_encrypted_record_tag() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];
        let tags_json = CString::new(r##"{"tag1": "value1"}"##).unwrap();

        let err = add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let new_tags_json = CString::new(r##"{"tag11": "value1"}"##).unwrap();
        let err = update_record_tags(handle, type_.as_ptr(), id.as_ptr(), new_tags_json.as_ptr());
        assert_eq!(err, ErrorCode::UnknownTag);

        let err = delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_update_encrypted_record_tag_value_too_long() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];
        let tags_json = CString::new(r##"{"~tag1": "value1"}"##).unwrap();

        let err = add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let long_tag_value = random_string(2817);
        let new_tags_json = json!({"~tag1": long_tag_value}).to_string();
        let new_tags_json = CString::new(new_tags_json).unwrap();

        let err = update_record_tags(handle, type_.as_ptr(), id.as_ptr(), new_tags_json.as_ptr());
        assert_eq!(err, ErrorCode::TagDataTooLong);

        let err = delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_update_plaintext_record_tag_value_too_long() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];
        let tags_json = CString::new(r##"{"~tag1": "value1"}"##).unwrap();

        let err = add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let long_tag_value = random_string(2817);
        let new_tags_json = json!({"~tag1": long_tag_value}).to_string();
        let new_tags_json = CString::new(new_tags_json).unwrap();

        let err = update_record_tags(handle, type_.as_ptr(), id.as_ptr(), new_tags_json.as_ptr());
        assert_eq!(err, ErrorCode::TagDataTooLong);

        let err = delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_update_unknown_plaintext_record_tag() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];
        let tags_json = CString::new(r##"{"~tag1": "value1"}"##).unwrap();

        let err = add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let new_tags_json = CString::new(r##"{"~tag11": "value1"}"##).unwrap();
        let err = update_record_tags(handle, type_.as_ptr(), id.as_ptr(), new_tags_json.as_ptr());
        assert_eq!(err, ErrorCode::UnknownTag);

        let err = delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_update_record_tag_unknown_record() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();

        let tags_json = CString::new(r##"{"~tag1": "value1"}"##).unwrap();
        let err = update_record_tags(handle, type_.as_ptr(), id.as_ptr(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::UnknownRecord);
    }

    #[test]
    fn test_update_record_tag_invalid_handle() {
        let handle = -1;

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();

        let tags_json = CString::new(r##"{"~tag11": "value1"}"##).unwrap();
        let err = update_record_tags(handle, type_.as_ptr(), id.as_ptr(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::InvalidStorageHandle);
    }

    /** Storage DELETE_RECORD_TAGS Tests */

    #[test]
    fn test_delete_record_tags() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];
        let tags_json = CString::new(r##"{"tag1": "value1", "tag2": "value2", "~tag3": "value3"}"##).unwrap();

        let err = add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let tag_names = CString::new(r##"["tag1", "tag2", "~tag3"]"##).unwrap();
        let err = delete_record_tags(handle, type_.as_ptr(), id.as_ptr(), tag_names.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let mut record_handle = -1;
        let options_json = CString::new(r##"{"fetch_value": true, "fetch_tags": true}"##).unwrap();
        let err = get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let mut tags_json_p: *const c_char = ptr::null_mut();
        let err = get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::Success);

        let tags_json_empty = CString::new(r##"{}"##).unwrap();
        let expected_tags_map: HashMap<String, String> = serde_json::from_slice(tags_json_empty.as_bytes()).unwrap();
        let tags_map: HashMap<String, String> = serde_json::from_str(unsafe{CStr::from_ptr(tags_json_p)}.to_str().unwrap()).unwrap();
        assert_eq!(tags_map, expected_tags_map);

        let err = delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_delete_unknown_tags() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];
        let tags_json = CString::new(r##"{"tag1": "value1", "tag2": "value2", "~tag3": "value3"}"##).unwrap();

        let err = add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let tag_names = CString::new(r##"["tag11", "tag22", "~tag33"]"##).unwrap();
        let err = delete_record_tags(handle, type_.as_ptr(), id.as_ptr(), tag_names.as_ptr());
        assert_eq!(err, ErrorCode::UnknownTag);

        let err = delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_delete_tags_for_unknown_record() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let tag_names = CString::new(r##"["tag1", "tag2", "~tag3"]"##).unwrap();

        let err = delete_record_tags(handle, type_.as_ptr(), id.as_ptr(), tag_names.as_ptr());
        assert_eq!(err, ErrorCode::UnknownRecord);
    }

    #[test]
    fn test_delete_unknown_tags_invalid_handle() {
        let handle = -1;

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();

        let tag_names = CString::new(r##"["tag11", "tag22", "~tag33"]"##).unwrap();
        let err = delete_record_tags(handle, type_.as_ptr(), id.as_ptr(), tag_names.as_ptr());
        assert_eq!(err, ErrorCode::InvalidStorageHandle);
    }
}

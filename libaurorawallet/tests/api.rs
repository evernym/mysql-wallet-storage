// Local dependencies
extern crate libaurorawallet;

use libaurorawallet::api as api;
use libaurorawallet::errors::error_code::ErrorCode;

mod test_utils;
use test_utils::config::{ConfigType, Config};
use test_utils::helper_functions::{random_string, random_name};

// External dependencies
extern crate libc;
use libc::c_char;
use std::collections::HashMap;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate lazy_static;

use std::ptr;
use std::ffi::{CStr, CString};
use std::slice;

mod high_casees {

    use super::*;

    lazy_static! {
        static ref TEST_CONFIG: Config = Config::new(ConfigType::QA);
    }

    fn open_storage() -> i32 {
        let name = CString::new("test-wallet").unwrap();
        let config = CString::new(TEST_CONFIG.get_config()).unwrap();
        let runtime_config = CString::new(TEST_CONFIG.get_runtime_config()).unwrap();
        let credentials = CString::new(TEST_CONFIG.get_credentials()).unwrap();
        let mut handle: i32 = -1;

        let err = api::open(name.as_ptr(), config.as_ptr(), runtime_config.as_ptr(), credentials.as_ptr(), &mut handle);

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
        let config = CString::new(TEST_CONFIG.get_config()).unwrap();
        let runtime_config = CString::new(TEST_CONFIG.get_runtime_config()).unwrap();
        let credentials = CString::new(TEST_CONFIG.get_credentials()).unwrap();

        let err = api::create(name.as_ptr(), config.as_ptr(), credentials.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let mut handle: i32 = -1;
        let err = api::open(name.as_ptr(), config.as_ptr(), runtime_config.as_ptr(), credentials.as_ptr(), &mut handle);
        assert_eq!(err, ErrorCode::Success);

        let err = api::close(handle);
        assert_eq!(err, ErrorCode::Success);

        let err = api::delete(name.as_ptr(), config.as_ptr(), credentials.as_ptr());
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

        let err = api::add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = api::get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = api::get_record_id(handle, record_handle, &mut id_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), id);

        let err = api::get_record_value(handle, record_handle, &mut value_p, &mut value_len_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe { slice::from_raw_parts(value_p, value_len_p) }, value.as_slice());

        let err = api::get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::Success);

        let expected_tags_map: HashMap<String, String> = serde_json::from_slice(tags_json.as_bytes()).unwrap();
        let tags_map: HashMap<String, String> = serde_json::from_str(unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap()).unwrap();
        assert_eq!(tags_map, expected_tags_map);

        let err = api::delete_record(handle, type_.as_ptr(), id.as_ptr());
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

        let err = api::add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = api::get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = api::get_record_id(handle, record_handle, &mut id_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), id);

        let err = api::get_record_value(handle, record_handle, &mut value_p, &mut value_len_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe { slice::from_raw_parts(value_p, value_len_p) }, value.as_slice());

        let err = api::get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::TagsNotFetched);

        let err = api::delete_record(handle, type_.as_ptr(), id.as_ptr());
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

        let err = api::add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = api::get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = api::get_record_id(handle, record_handle, &mut id_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), id);

        let err = api::get_record_value(handle, record_handle, &mut value_p, &mut value_len_p);
        assert_eq!(err, ErrorCode::ValueNotFetched);

        let err = api::get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::Success);

        let expected_tags_map: HashMap<String, String> = serde_json::from_slice(tags_json.as_bytes()).unwrap();
        let tags_map: HashMap<String, String> = serde_json::from_str(unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap()).unwrap();
        assert_eq!(tags_map, expected_tags_map);

        let err = api::delete_record(handle, type_.as_ptr(), id.as_ptr());
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

        let err = api::add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = api::get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = api::get_record_id(handle, record_handle, &mut id_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), id);

        let err = api::get_record_value(handle, record_handle, &mut value_p, &mut value_len_p);
        assert_eq!(err, ErrorCode::ValueNotFetched);

        let err = api::get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::TagsNotFetched);

        let err = api::delete_record(handle, type_.as_ptr(), id.as_ptr());
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

        let err = api::add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = api::get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = api::get_record_id(handle, record_handle, &mut id_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), id);

        let err = api::get_record_value(handle, record_handle, &mut value_p, &mut value_len_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe { slice::from_raw_parts(value_p, value_len_p) }, value.as_slice());

        let err = api::get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::Success);

        let expected_tags_map: HashMap<String, String> = serde_json::from_slice(tags_json.as_bytes()).unwrap();
        let tags_map: HashMap<String, String> = serde_json::from_str(unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap()).unwrap();
        assert_eq!(tags_map, expected_tags_map);

        let err = api::delete_record(handle, type_.as_ptr(), id.as_ptr());
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

        let err = api::add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = api::get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = api::get_record_id(handle, record_handle, &mut id_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), id);

        let err = api::get_record_value(handle, record_handle, &mut value_p, &mut value_len_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe { slice::from_raw_parts(value_p, value_len_p) }, value.as_slice());

        let err = api::get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::Success);

        let expected_tags_map: HashMap<String, String> = serde_json::from_slice(tags_json.as_bytes()).unwrap();
        let tags_map: HashMap<String, String> = serde_json::from_str(unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap()).unwrap();
        assert_eq!(tags_map, expected_tags_map);

        let err = api::delete_record(handle, type_.as_ptr(), id.as_ptr());
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

        let err = api::add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = api::get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = api::get_record_id(handle, record_handle, &mut id_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), id);

        let err = api::get_record_value(handle, record_handle, &mut value_p, &mut value_len_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe { slice::from_raw_parts(value_p, value_len_p) }, value.as_slice());

        let err = api::get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::TagsNotFetched);

        let err = api::delete_record(handle, type_.as_ptr(), id.as_ptr());
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

        let err = api::add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = api::get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = api::get_record_id(handle, record_handle, &mut id_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), id);

        let err = api::get_record_value(handle, record_handle, &mut value_p, &mut value_len_p);
        assert_eq!(err, ErrorCode::ValueNotFetched);

        let err = api::get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::Success);

        let expected_tags_map: HashMap<String, String> = serde_json::from_slice(tags_json.as_bytes()).unwrap();
        let tags_map: HashMap<String, String> = serde_json::from_str(unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap()).unwrap();
        assert_eq!(tags_map, expected_tags_map);

        let err = api::delete_record(handle, type_.as_ptr(), id.as_ptr());
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

        let err = api::add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = api::get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = api::get_record_id(handle, record_handle, &mut id_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), id);

        let err = api::get_record_value(handle, record_handle, &mut value_p, &mut value_len_p);
        assert_eq!(err, ErrorCode::ValueNotFetched);

        let err = api::get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::TagsNotFetched);

        let err = api::delete_record(handle, type_.as_ptr(), id.as_ptr());
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

        let err = api::add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = api::get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = api::get_record_id(handle, record_handle, &mut id_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), id);

        let err = api::get_record_value(handle, record_handle, &mut value_p, &mut value_len_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe { slice::from_raw_parts(value_p, value_len_p) }, value.as_slice());

        let err = api::get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::Success);

        let expected_tags_map: HashMap<String, String> = serde_json::from_slice(tags_json.as_bytes()).unwrap();
        let tags_map: HashMap<String, String> = serde_json::from_str(unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap()).unwrap();
        assert_eq!(tags_map, expected_tags_map);

        let err = api::delete_record(handle, type_.as_ptr(), id.as_ptr());
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

        let err = api::add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = api::get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = api::get_record_id(handle, record_handle, &mut id_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), id);

        let err = api::get_record_value(handle, record_handle, &mut value_p, &mut value_len_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe { slice::from_raw_parts(value_p, value_len_p) }, value.as_slice());

        let err = api::get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::Success);

        let expected_tags_map: HashMap<String, String> = serde_json::from_slice(tags_json.as_bytes()).unwrap();
        let tags_map: HashMap<String, String> = serde_json::from_str(unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap()).unwrap();
        assert_eq!(tags_map, expected_tags_map);

        let err = api::add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::RecordAlreadExists);

        let err = api::delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_delete_unknown() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();

        let err = api::delete_record(handle, type_.as_ptr(), id.as_ptr());
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
        let err = api::add_record(handle, type_.as_ptr(), id.as_ptr(), initial_value.as_ptr(), initial_value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let new_value = vec![2, 5, 8, 13];
        let err = api::update_record_value(handle, type_.as_ptr(), id.as_ptr(), new_value.as_ptr(), new_value.len());
        assert_eq!(err, ErrorCode::Success);

        let mut record_handle = -1;
        let options_json = CString::new(r##"{"fetch_value": true, "fetch_tags": false}"##).unwrap();
        let err = api::get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let mut value_p: *const u8 = ptr::null_mut();
        let mut value_len_p = 0;
        let err = api::get_record_value(handle, record_handle, &mut value_p, &mut value_len_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe { slice::from_raw_parts(value_p, value_len_p) }, new_value.as_slice());

        let err = api::delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_update_record_value_with_same_value() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let tags_json = CString::new(r##"{"tag1": "value1", "tag2": "value2", "~tag3": "value3"}"##).unwrap();

        let initial_value = vec![1, 2, 3, 4];
        let err = api::add_record(handle, type_.as_ptr(), id.as_ptr(), initial_value.as_ptr(), initial_value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let new_value = vec![1, 2, 3, 4];
        let err = api::update_record_value(handle, type_.as_ptr(), id.as_ptr(), new_value.as_ptr(), new_value.len());
        assert_eq!(err, ErrorCode::Success);

        let mut record_handle = -1;
        let options_json = CString::new(r##"{"fetch_value": true, "fetch_tags": false}"##).unwrap();
        let err = api::get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let mut value_p: *const u8 = ptr::null_mut();
        let mut value_len_p = 0;
        let err = api::get_record_value(handle, record_handle, &mut value_p, &mut value_len_p);
        assert_eq!(err, ErrorCode::Success);
        assert_eq!(unsafe { slice::from_raw_parts(value_p, value_len_p) }, new_value.as_slice());

        let err = api::delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_update_value_for_invalid_handle() {
        let handle = -1;

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let new_value = vec![1, 2, 3, 4];

        let err = api::update_record_value(handle, type_.as_ptr(), id.as_ptr(), new_value.as_ptr(), new_value.len());
        assert_eq!(err, ErrorCode::InvalidStorageHandle);
    }

    #[test]
    fn test_update_value_for_unknown_record() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let new_value = vec![1, 2, 3, 4];

        let err = api::update_record_value(handle, type_.as_ptr(), id.as_ptr(), new_value.as_ptr(), new_value.len());
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
        let err = api::add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json_empty.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let tags_json = CString::new(r##"{"tag1": "value1", "tag2": "value2", "~tag3": "value3"}"##).unwrap();
        let err = api::add_record_tags(handle, type_.as_ptr(), id.as_ptr(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let mut record_handle = -1;
        let options_json = CString::new(r##"{"fetch_value": true, "fetch_tags": true}"##).unwrap();
        let err = api::get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let mut tags_json_p: *const c_char = ptr::null_mut();
        let err = api::get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::Success);

        let expected_tags_map: HashMap<String, String> = serde_json::from_slice(tags_json.as_bytes()).unwrap();
        let tags_map: HashMap<String, String> = serde_json::from_str(unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap()).unwrap();
        assert_eq!(tags_map, expected_tags_map);

        let err = api::delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_add_duplicate_record_tags() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];

        let tags_json_empty = CString::new(r##"{}"##).unwrap();
        let err = api::add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json_empty.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let tags_json = CString::new(r##"{"tag1": "value1", "tag2": "value2", "~tag3": "value3"}"##).unwrap();
        let err = api::add_record_tags(handle, type_.as_ptr(), id.as_ptr(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = api::add_record_tags(handle, type_.as_ptr(), id.as_ptr(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::TagAlreadyExists);

        let mut record_handle = -1;
        let options_json = CString::new(r##"{"fetch_value": true, "fetch_tags": true}"##).unwrap();
        let err = api::get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let mut tags_json_p: *const c_char = ptr::null_mut();
        let err = api::get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::Success);

        let expected_tags_map: HashMap<String, String> = serde_json::from_slice(tags_json.as_bytes()).unwrap();
        let tags_map: HashMap<String, String> = serde_json::from_str(unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap()).unwrap();
        assert_eq!(tags_map, expected_tags_map);

        let err = api::delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_add_encrypted_tags_tag_name_too_long() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];

        let tags_json_empty = CString::new(r##"{}"##).unwrap();
        let err = api::add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json_empty.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let long_tag_name = random_string(257);
        let tags_json = json!({long_tag_name: "tag_value"}).to_string();
        let tags_json = CString::new(tags_json).unwrap();

        let err = api::add_record_tags(handle, type_.as_ptr(), id.as_ptr(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::TagDataTooLong);

        let err = api::delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_add_plaintext_tags_tag_name_too_long() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];

        let tags_json_empty = CString::new(r##"{}"##).unwrap();
        let err = api::add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json_empty.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let mut long_tag_name = random_string(257);
        long_tag_name.push('~');
        let tags_json = json!({long_tag_name: "tag_value"}).to_string();
        let tags_json = CString::new(tags_json).unwrap();

        let err = api::add_record_tags(handle, type_.as_ptr(), id.as_ptr(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::TagDataTooLong);

        let err = api::delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_add_encrypted_tags_tag_value_too_long() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];

        let tags_json_empty = CString::new(r##"{}"##).unwrap();
        let err = api::add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json_empty.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let long_tag_value = random_string(2817);
        let tags_json = json!({"tag_name": long_tag_value}).to_string();
        let tags_json = CString::new(tags_json).unwrap();

        let err = api::add_record_tags(handle, type_.as_ptr(), id.as_ptr(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::TagDataTooLong);

        let err = api::delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_add_plaintext_tags_tag_value_too_long() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];

        let tags_json_empty = CString::new(r##"{}"##).unwrap();
        let err = api::add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json_empty.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let long_tag_value = random_string(2817);
        let tags_json = json!({"~tag_name": long_tag_value}).to_string();
        let tags_json = CString::new(tags_json).unwrap();

        let err = api::add_record_tags(handle, type_.as_ptr(), id.as_ptr(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::TagDataTooLong);

        let err = api::delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_add_tags_for_invalid_handle() {
        let handle = -1;

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let tags_json = CString::new(r##"{"tag1": "value1", "tag2": "value2", "~tag3": "value3"}"##).unwrap();

        let err = api::add_record_tags(handle, type_.as_ptr(), id.as_ptr(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::InvalidStorageHandle);
    }

    #[test]
    fn test_add_tags_for_unknown_record() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let tags_json = CString::new(r##"{"tag1": "value1", "tag2": "value2", "~tag3": "value3"}"##).unwrap();

        let err = api::add_record_tags(handle, type_.as_ptr(), id.as_ptr(), tags_json.as_ptr());
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

        let err = api::add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let new_tags_json = CString::new(r##"{"tag1": "value1_new", "tag2": "value2_new", "~tag3": "value3_new"}"##).unwrap();
        let err = api::update_record_tags(handle, type_.as_ptr(), id.as_ptr(), new_tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let mut record_handle = -1;
        let options_json = CString::new(r##"{"fetch_value": true, "fetch_tags": true}"##).unwrap();
        let err = api::get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let mut tags_json_p: *const c_char = ptr::null_mut();
        let err = api::get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::Success);

        let expected_tags_map: HashMap<String, String> = serde_json::from_slice(new_tags_json.as_bytes()).unwrap();
        let tags_map: HashMap<String, String> = serde_json::from_str(unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap()).unwrap();
        assert_eq!(tags_map, expected_tags_map);

        let err = api::delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_update_record_encrypted_tags_with_same_values() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];
        let tags_json = CString::new(r##"{"tag1": "value1", "tag2": "value2"}"##).unwrap();

        let err = api::add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let new_tags_json = CString::new(r##"{"tag1": "value1", "tag2": "value2"}"##).unwrap();
        let err = api::update_record_tags(handle, type_.as_ptr(), id.as_ptr(), new_tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = api::delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_update_record_unencrypted_tags_with_same_values() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];
        let tags_json = CString::new(r##"{"~tag1": "value1", "~tag2": "value2"}"##).unwrap();

        let err = api::add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let new_tags_json = CString::new(r##"{"~tag1": "value1", "~tag2": "value2"}"##).unwrap();
        let err = api::update_record_tags(handle, type_.as_ptr(), id.as_ptr(), new_tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = api::delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_update_unknown_encrypted_record_tag() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];
        let tags_json = CString::new(r##"{"tag1": "value1"}"##).unwrap();

        let err = api::add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let new_tags_json = CString::new(r##"{"tag11": "value1"}"##).unwrap();
        let err = api::update_record_tags(handle, type_.as_ptr(), id.as_ptr(), new_tags_json.as_ptr());
        assert_eq!(err, ErrorCode::UnknownTag);

        let err = api::delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_update_encrypted_record_tag_value_too_long() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];
        let tags_json = CString::new(r##"{"~tag1": "value1"}"##).unwrap();

        let err = api::add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let long_tag_value = random_string(2817);
        let new_tags_json = json!({"~tag1": long_tag_value}).to_string();
        let new_tags_json = CString::new(new_tags_json).unwrap();

        let err = api::update_record_tags(handle, type_.as_ptr(), id.as_ptr(), new_tags_json.as_ptr());
        assert_eq!(err, ErrorCode::TagDataTooLong);

        let err = api::delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_update_plaintext_record_tag_value_too_long() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];
        let tags_json = CString::new(r##"{"~tag1": "value1"}"##).unwrap();

        let err = api::add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let long_tag_value = random_string(2817);
        let new_tags_json = json!({"~tag1": long_tag_value}).to_string();
        let new_tags_json = CString::new(new_tags_json).unwrap();

        let err = api::update_record_tags(handle, type_.as_ptr(), id.as_ptr(), new_tags_json.as_ptr());
        assert_eq!(err, ErrorCode::TagDataTooLong);

        let err = api::delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_update_unknown_plaintext_record_tag() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];
        let tags_json = CString::new(r##"{"~tag1": "value1"}"##).unwrap();

        let err = api::add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let new_tags_json = CString::new(r##"{"~tag11": "value1"}"##).unwrap();
        let err = api::update_record_tags(handle, type_.as_ptr(), id.as_ptr(), new_tags_json.as_ptr());
        assert_eq!(err, ErrorCode::UnknownTag);

        let err = api::delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_update_record_tag_unknown_record() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();

        let tags_json = CString::new(r##"{"~tag1": "value1"}"##).unwrap();
        let err = api::update_record_tags(handle, type_.as_ptr(), id.as_ptr(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::UnknownRecord);
    }

    #[test]
    fn test_update_record_tag_invalid_handle() {
        let handle = -1;

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();

        let tags_json = CString::new(r##"{"~tag11": "value1"}"##).unwrap();
        let err = api::update_record_tags(handle, type_.as_ptr(), id.as_ptr(), tags_json.as_ptr());
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

        let err = api::add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let tag_names = CString::new(r##"["tag1", "tag2", "~tag3"]"##).unwrap();
        let err = api::delete_record_tags(handle, type_.as_ptr(), id.as_ptr(), tag_names.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let mut record_handle = -1;
        let options_json = CString::new(r##"{"fetch_value": true, "fetch_tags": true}"##).unwrap();
        let err = api::get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let mut tags_json_p: *const c_char = ptr::null_mut();
        let err = api::get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::Success);

        let tags_json_empty = CString::new(r##"{}"##).unwrap();
        let expected_tags_map: HashMap<String, String> = serde_json::from_slice(tags_json_empty.as_bytes()).unwrap();
        let tags_map: HashMap<String, String> = serde_json::from_str(unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap()).unwrap();
        assert_eq!(tags_map, expected_tags_map);

        let err = api::delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_delete_unknown_tags() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let value = vec![1, 2, 3, 4];
        let tags_json = CString::new(r##"{"tag1": "value1", "tag2": "value2", "~tag3": "value3"}"##).unwrap();

        let err = api::add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let tag_names = CString::new(r##"["tag11", "tag22", "~tag33"]"##).unwrap();
        let err = api::delete_record_tags(handle, type_.as_ptr(), id.as_ptr(), tag_names.as_ptr());
        assert_eq!(err, ErrorCode::UnknownTag);

        let err = api::delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    #[test]
    fn test_delete_tags_for_unknown_record() {
        let handle = open_storage();

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();
        let tag_names = CString::new(r##"["tag1", "tag2", "~tag3"]"##).unwrap();

        let err = api::delete_record_tags(handle, type_.as_ptr(), id.as_ptr(), tag_names.as_ptr());
        assert_eq!(err, ErrorCode::UnknownRecord);
    }

    #[test]
    fn test_delete_unknown_tags_invalid_handle() {
        let handle = -1;

        let type_ = CString::new("type1").unwrap();
        let id = CString::new(random_name()).unwrap();

        let tag_names = CString::new(r##"["tag11", "tag22", "~tag33"]"##).unwrap();
        let err = api::delete_record_tags(handle, type_.as_ptr(), id.as_ptr(), tag_names.as_ptr());
        assert_eq!(err, ErrorCode::InvalidStorageHandle);
    }
}

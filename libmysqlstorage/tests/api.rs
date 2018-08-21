// Local dependencies
extern crate mysqlstorage;

use mysqlstorage::api as api;
use mysqlstorage::errors::error_code::ErrorCode;
use mysqlstorage::utils::logger;

mod test_utils;
use test_utils::config::Config;
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
        static ref TEST_CONFIG: Config = Config::new();
    }

    struct TestWallet {
        handle: i32,
        is_mock: bool,
        name: CString,
        config: CString,
        credentials: CString,
        metadata: CString
    }

    impl TestWallet {
        ///
        ///  is_mock: do we want just an object, or want to have a wallet in the DB. If not mock
        ///  wallet will be created, opened, and ready for tests
        ///
        fn new_default(is_mock: bool) -> Self {

            logger::init();

            let handle = -1;
            let name = CString::new(random_name()).unwrap();
            let config = CString::new(TEST_CONFIG.get_config()).unwrap();
            let credentials = CString::new(TEST_CONFIG.get_credentials()).unwrap();
            let metadata = CString::new(random_string(100)).unwrap();

            let mut test_wallet = TestWallet{handle, is_mock, name, config, credentials, metadata};

            if !is_mock {
                test_wallet._create();
                let wallet_handle = test_wallet._open();
                test_wallet.handle = wallet_handle;
            }

            test_wallet
        }

        fn _create(&self) {
            let err = api::create_storage(self.name.as_ptr(), self.config.as_ptr(), self.credentials.as_ptr(), self.metadata.as_ptr());
            assert_eq!(err, ErrorCode::Success);
        }

        fn _open(&mut self) -> i32 {
            let mut handle: i32 = -1;
            let err = api::open_storage(self.name.as_ptr(), self.config.as_ptr(), self.credentials.as_ptr(), &mut handle);
            assert_eq!(err, ErrorCode::Success);

            handle
        }

        fn add_record(&self, record: &TestRecord) {
            let err = api::add_record(self.handle, record.type_.as_ptr(), record.id.as_ptr(), record.value.as_ptr(), record.value.len(), record.tags_json.as_ptr());
            assert_eq!(err, ErrorCode::Success);
        }
    }

    impl Drop for TestWallet {
        fn drop(&mut self) {
            if !self.is_mock {
                api::delete_storage(self.name.as_ptr(), self.config.as_ptr(), self.credentials.as_ptr());
            }
        }
    }

    struct TestRecord {
        type_: CString,
        id: CString,
        value: Vec<u8>,
        tags_json: CString,
        tag_names: CString
    }

    impl TestRecord {
        fn new_default(with_tags: bool) -> Self {
            let type_ = CString::new(random_string(10)).unwrap();
            let id = CString::new(random_name()).unwrap();
            let value = vec![1, 2, 3, 4];
            let tags_json;
            let tag_names;

            if with_tags {
                tags_json = CString::new(r#"{"tag1": "value1", "tag2": "value2", "~tag3": "value3"}"#).unwrap();
                tag_names = CString::new(r#"["tag1", "tag2", "~tag3"]"#).unwrap();
            }
            else {
                tags_json = CString::new(r#"{}"#).unwrap();
                tag_names = CString::new(r#"[]"#).unwrap();
            }

            TestRecord{type_, id, value, tags_json, tag_names}
        }
    }

    fn fetch_options(retrieve_type: bool, retrieve_value: bool, retrieve_tags: bool) -> CString {
        let mut map = HashMap::new();
        map.insert("retrieveType", retrieve_type);
        map.insert("retrieveValue", retrieve_value);
        map.insert("retrieveTags", retrieve_tags);

        CString::new(serde_json::to_string(&map).unwrap()).unwrap()
    }

    fn search_options(retrieve_records: bool, retrieve_total_count: bool, retrieve_type: bool, retrieve_value: bool, retrieve_tags: bool) -> CString {
        let mut map = HashMap::new();

        map.insert("retrieveRecords", retrieve_records);
        map.insert("retrieveTotalCount", retrieve_total_count);
        map.insert("retrieveType", retrieve_type);
        map.insert("retrieveValue", retrieve_value);
        map.insert("retrieveTags", retrieve_tags);

        CString::new(serde_json::to_string(&map).unwrap()).unwrap()
    }

    /** Storage CREATE */

    #[test]
    fn test_create() {
        let wallet = TestWallet::new_default(true);

        println!("NEMANJAAAA Config: {:?}, Credentials {:?}", wallet.config, wallet.credentials);

        let err = api::create_storage(wallet.name.as_ptr(), wallet.config.as_ptr(), wallet.credentials.as_ptr(), wallet.metadata.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = api::delete_storage(wallet.name.as_ptr(), wallet.config.as_ptr(), wallet.credentials.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }
//
//    #[test]
//    fn test_create_wallet_already_exists() {
//        let wallet = TestWallet::new_default(false);
//
//        let err = api::create_storage(wallet.name.as_ptr(), wallet.config.as_ptr(), wallet.credentials.as_ptr(), wallet.metadata.as_ptr());
//        assert_eq!(err, ErrorCode::WalletAlreadyExistsError);
//    }
//
//    #[test]
//    fn test_create_bad_config_format() {
//        let wallet = TestWallet::new_default(true);
//
//        let bad_config = CString::new("..").unwrap();
//
//        let err = api::create_storage(wallet.name.as_ptr(), bad_config.as_ptr(), wallet.credentials.as_ptr(), wallet.metadata.as_ptr());
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_create_null_config() {
//        let wallet = TestWallet::new_default(true);
//
//        let err = api::create_storage(wallet.name.as_ptr(), std::ptr::null(), wallet.credentials.as_ptr(), wallet.metadata.as_ptr());
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_create_bad_credentials_format() {
//        let wallet = TestWallet::new_default(true);
//
//        let bad_credentials = CString::new("...").unwrap();
//
//        let err = api::create_storage(wallet.name.as_ptr(), wallet.config.as_ptr(), bad_credentials.as_ptr(), wallet.metadata.as_ptr());
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_create_null_credentials() {
//        let wallet = TestWallet::new_default(true);
//
//        let err = api::create_storage(wallet.name.as_ptr(), wallet.config.as_ptr(), std::ptr::null(), wallet.metadata.as_ptr());
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_create_bad_data_in_config() {
//        let wallet = TestWallet::new_default(true);
//
//        let bad_config = CString::new(r#"{"key": "value"}"#).unwrap();
//
//        let err = api::create_storage(wallet.name.as_ptr(), bad_config.as_ptr(), wallet.credentials.as_ptr(), wallet.metadata.as_ptr());
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_create_bad_data_in_credentials() {
//        let wallet = TestWallet::new_default(true);
//
//        let bad_credentials = CString::new(r#"{"key": "value"}"#).unwrap();
//
//        let err = api::create_storage(wallet.name.as_ptr(), wallet.config.as_ptr(), bad_credentials.as_ptr(), wallet.metadata.as_ptr());
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_create_with_large_name() {
//        let wallet = TestWallet::new_default(true);
//
//        let big_name = CString::new(random_string(2046)).unwrap();
//
//        let err = api::create_storage(big_name.as_ptr(), wallet.config.as_ptr(), wallet.credentials.as_ptr(), wallet.metadata.as_ptr());
//        assert_eq!(err, ErrorCode::IOError);
//    }
//
//    /** Storage DELETE */
//
//    #[test]
//    fn test_delete_empty() {
//        let wallet = TestWallet::new_default(false);
//
//        let err = api::delete_storage(wallet.name.as_ptr(), wallet.config.as_ptr(), wallet.credentials.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_delete_non_empty() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(true);
//
//        wallet.add_record(&record);
//
//        let err = api::delete_storage(wallet.name.as_ptr(), wallet.config.as_ptr(), wallet.credentials.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_delete_invalid_wallet_name() {
//        let wallet = TestWallet::new_default(true);
//
//        let err = api::delete_storage(wallet.name.as_ptr(), wallet.config.as_ptr(), wallet.credentials.as_ptr());
//        assert_eq!(err, ErrorCode::InvalidState);
//    }
//
//    #[test]
//    fn test_delete_bad_config_format() {
//        let wallet = TestWallet::new_default(true);
//
//        let bad_config = CString::new("..").unwrap();
//
//        let err = api::delete_storage(wallet.name.as_ptr(), bad_config.as_ptr(), wallet.credentials.as_ptr());
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_delete_null_config() {
//        let wallet = TestWallet::new_default(true);
//
//        let err = api::delete_storage(wallet.name.as_ptr(), std::ptr::null(), wallet.credentials.as_ptr());
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_delete_bad_credentials_format() {
//        let wallet = TestWallet::new_default(true);
//
//        let bad_credentials = CString::new("..").unwrap();
//
//        let err = api::delete_storage(wallet.name.as_ptr(), wallet.config.as_ptr(), bad_credentials.as_ptr());
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_delete_null_credentials() {
//        let wallet = TestWallet::new_default(true);
//
//        let err = api::delete_storage(wallet.name.as_ptr(), wallet.config.as_ptr(), std::ptr::null());
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_delete_bad_data_in_config() {
//        let wallet = TestWallet::new_default(true);
//
//        let bad_config = CString::new(r#"{"key": "value"}"#).unwrap();
//
//        let err = api::delete_storage(wallet.name.as_ptr(), bad_config.as_ptr(), wallet.credentials.as_ptr());
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_delete_bad_data_in_credentials() {
//        let wallet = TestWallet::new_default(true);
//
//        let bad_credentials = CString::new(r#"{"key": "value"}"#).unwrap();
//
//        let err = api::delete_storage(wallet.name.as_ptr(), wallet.config.as_ptr(), bad_credentials.as_ptr());
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    /** Storage OPEN Tests */
//
//    #[test]
//    fn test_open() {
//        let wallet = TestWallet::new_default(false);
//
//        let mut handle: i32 = -1;
//
//        let err = api::open_storage(wallet.name.as_ptr(), wallet.config.as_ptr(), wallet.credentials.as_ptr(), &mut handle);
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_open_invalid_wallet_name() {
//        let wallet = TestWallet::new_default(true);
//
//        let mut handle: i32 = -1;
//
//        let err = api::open_storage(wallet.name.as_ptr(), wallet.config.as_ptr(), wallet.credentials.as_ptr(), &mut handle);
//        assert_eq!(err, ErrorCode::InvalidState);
//    }
//
//    #[test]
//    fn test_open_bad_config_format() {
//        let wallet = TestWallet::new_default(true);
//
//        let bad_config = CString::new("..").unwrap();
//        let mut handle: i32 = -1;
//
//        let err = api::open_storage(wallet.name.as_ptr(), bad_config.as_ptr(), wallet.credentials.as_ptr(), &mut handle);
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_open_null_config() {
//        let wallet = TestWallet::new_default(true);
//
//        let mut handle: i32 = -1;
//
//        let err = api::open_storage(wallet.name.as_ptr(), std::ptr::null(), wallet.credentials.as_ptr(), &mut handle);
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_open_bad_credentials_format() {
//        let wallet = TestWallet::new_default(true);
//
//        let bad_credentials = CString::new("..").unwrap();
//        let mut handle: i32 = -1;
//
//        let err = api::open_storage(wallet.name.as_ptr(), wallet.config.as_ptr(), bad_credentials.as_ptr(), &mut handle);
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_open_null_credentials() {
//        let wallet = TestWallet::new_default(true);
//
//        let mut handle: i32 = -1;
//
//        let err = api::open_storage(wallet.name.as_ptr(), wallet.config.as_ptr(), std::ptr::null(), &mut handle);
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_open_bad_data_in_config() {
//        let wallet = TestWallet::new_default(true);
//
//        let bad_config = CString::new(r#"{"key": "value"}"#).unwrap();
//        let mut handle: i32 = -1;
//
//        let err = api::open_storage(wallet.name.as_ptr(), bad_config.as_ptr(), wallet.credentials.as_ptr(), &mut handle);
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_open_bad_data_in_credentials() {
//        let wallet = TestWallet::new_default(true);
//
//        let bad_credentials = CString::new(r#"{"key": "value"}"#).unwrap();
//        let mut handle: i32 = -1;
//
//        let err = api::open_storage(wallet.name.as_ptr(), wallet.config.as_ptr(), bad_credentials.as_ptr(), &mut handle);
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    /** Storage ADD_RECORD, GET_RECORD, DELETE_RECORD Tests WITH TAGS */
//
//    #[test]
//    fn test_add_record_with_tags_then_fetch_all() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(true);
//
//        let options_json = fetch_options(true, true, true);
//        let mut record_handle = -1;
//        let mut id_p: *const c_char = ptr::null_mut();
//        let mut value_p: *const u8 = ptr::null_mut();
//        let mut value_len_p = 0;
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let mut type_p: *const c_char = ptr::null_mut();
//
//        let err = api::add_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.value.as_ptr(), record.value.len(), record.tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), options_json.as_ptr(), &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record_id(wallet.handle, record_handle, &mut id_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), record.id);
//
//        let err = api::get_record_type(wallet.handle, record_handle, &mut type_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(type_p) }.to_owned(), record.type_);
//
//        let err = api::get_record_value(wallet.handle, record_handle, &mut value_p, &mut value_len_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { slice::from_raw_parts(value_p, value_len_p) }, record.value.as_slice());
//
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::Success);
//
//        let expected_tags_map: HashMap<String, serde_json::Value> = serde_json::from_slice(record.tags_json.as_bytes()).unwrap();
//        let tags_map: HashMap<String, serde_json::Value> = serde_json::from_str(unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap()).unwrap();
//        assert_eq!(tags_map, expected_tags_map);
//
//        let err = api::free_record(wallet.handle, record_handle);
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_add_record_with_tags_then_fetch_only_type() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(true);
//
//        let options_json = fetch_options(true, false, false);
//        let mut record_handle = -1;
//        let mut id_p: *const c_char = ptr::null_mut();
//        let mut value_p: *const u8 = ptr::null_mut();
//        let mut value_len_p = 0;
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let mut type_p: *const c_char = ptr::null_mut();
//
//        let err = api::add_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.value.as_ptr(), record.value.len(), record.tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), options_json.as_ptr(), &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record_id(wallet.handle, record_handle, &mut id_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), record.id);
//
//        let err = api::get_record_type(wallet.handle, record_handle, &mut type_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(type_p) }.to_owned(), record.type_);
//
//        let err = api::get_record_value(wallet.handle, record_handle, &mut value_p, &mut value_len_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let err = api::free_record(wallet.handle, record_handle);
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_add_record_with_tags_then_fetch_only_value() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(true);
//
//        let options_json = fetch_options(false, true, false);
//        let mut record_handle = -1;
//        let mut id_p: *const c_char = ptr::null_mut();
//        let mut value_p: *const u8 = ptr::null_mut();
//        let mut value_len_p = 0;
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let mut type_p: *const c_char = ptr::null_mut();
//
//        let err = api::add_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.value.as_ptr(), record.value.len(), record.tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), options_json.as_ptr(), &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record_id(wallet.handle, record_handle, &mut id_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), record.id);
//
//        let err = api::get_record_type(wallet.handle, record_handle, &mut type_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let err = api::get_record_value(wallet.handle, record_handle, &mut value_p, &mut value_len_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { slice::from_raw_parts(value_p, value_len_p) }, record.value.as_slice());
//
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let err = api::free_record(wallet.handle, record_handle);
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_add_record_with_tags_then_fetch_only_tags() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(true);
//
//        let options_json = fetch_options(false, false, true);
//        let mut record_handle = -1;
//        let mut id_p: *const c_char = ptr::null_mut();
//        let mut value_p: *const u8 = ptr::null_mut();
//        let mut value_len_p = 0;
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let mut type_p: *const c_char = ptr::null_mut();
//
//        let err = api::add_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.value.as_ptr(), record.value.len(), record.tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), options_json.as_ptr(), &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record_id(wallet.handle, record_handle, &mut id_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), record.id);
//
//        let err = api::get_record_type(wallet.handle, record_handle, &mut type_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let err = api::get_record_value(wallet.handle, record_handle, &mut value_p, &mut value_len_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::Success);
//
//        let expected_tags_map: HashMap<String, serde_json::Value> = serde_json::from_slice(record.tags_json.as_bytes()).unwrap();
//        let tags_map: HashMap<String, serde_json::Value> = serde_json::from_str(unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap()).unwrap();
//        assert_eq!(tags_map, expected_tags_map);
//
//        let err = api::free_record(wallet.handle, record_handle);
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_add_record_with_tags_then_fetch_type_value() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(true);
//
//        let options_json = fetch_options(true, true, false);
//        let mut record_handle = -1;
//        let mut id_p: *const c_char = ptr::null_mut();
//        let mut value_p: *const u8 = ptr::null_mut();
//        let mut value_len_p = 0;
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let mut type_p: *const c_char = ptr::null_mut();
//
//        let err = api::add_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.value.as_ptr(), record.value.len(), record.tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), options_json.as_ptr(), &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record_id(wallet.handle, record_handle, &mut id_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), record.id);
//
//        let err = api::get_record_type(wallet.handle, record_handle, &mut type_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(type_p) }.to_owned(), record.type_);
//
//        let err = api::get_record_value(wallet.handle, record_handle, &mut value_p, &mut value_len_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { slice::from_raw_parts(value_p, value_len_p) }, record.value.as_slice());
//
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let err = api::free_record(wallet.handle, record_handle);
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_add_record_with_tags_then_fetch_type_tags() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(true);
//
//        let options_json = fetch_options(true, false, true);
//        let mut record_handle = -1;
//        let mut id_p: *const c_char = ptr::null_mut();
//        let mut value_p: *const u8 = ptr::null_mut();
//        let mut value_len_p = 0;
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let mut type_p: *const c_char = ptr::null_mut();
//
//        let err = api::add_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.value.as_ptr(), record.value.len(), record.tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), options_json.as_ptr(), &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record_id(wallet.handle, record_handle, &mut id_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), record.id);
//
//        let err = api::get_record_type(wallet.handle, record_handle, &mut type_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(type_p) }.to_owned(), record.type_);
//
//        let err = api::get_record_value(wallet.handle, record_handle, &mut value_p, &mut value_len_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::Success);
//
//        let expected_tags_map: HashMap<String, serde_json::Value> = serde_json::from_slice(record.tags_json.as_bytes()).unwrap();
//        let tags_map: HashMap<String, serde_json::Value> = serde_json::from_str(unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap()).unwrap();
//        assert_eq!(tags_map, expected_tags_map);
//
//        let err = api::free_record(wallet.handle, record_handle);
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_add_record_with_tags_then_fetch_value_tags() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(true);
//
//        let options_json = fetch_options(false, true, true);
//        let mut record_handle = -1;
//        let mut id_p: *const c_char = ptr::null_mut();
//        let mut value_p: *const u8 = ptr::null_mut();
//        let mut value_len_p = 0;
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let mut type_p: *const c_char = ptr::null_mut();
//
//        let err = api::add_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.value.as_ptr(), record.value.len(), record.tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), options_json.as_ptr(), &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record_id(wallet.handle, record_handle, &mut id_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), record.id);
//
//        let err = api::get_record_type(wallet.handle, record_handle, &mut type_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let err = api::get_record_value(wallet.handle, record_handle, &mut value_p, &mut value_len_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { slice::from_raw_parts(value_p, value_len_p) }, record.value.as_slice());
//
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::Success);
//
//        let expected_tags_map: HashMap<String, serde_json::Value> = serde_json::from_slice(record.tags_json.as_bytes()).unwrap();
//        let tags_map: HashMap<String, serde_json::Value> = serde_json::from_str(unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap()).unwrap();
//        assert_eq!(tags_map, expected_tags_map);
//
//        let err = api::free_record(wallet.handle, record_handle);
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_add_record_with_tags_then_fetch_none() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(true);
//
//        let options_json = fetch_options(false, false, false);
//        let mut record_handle = -1;
//        let mut id_p: *const c_char = ptr::null_mut();
//        let mut value_p: *const u8 = ptr::null_mut();
//        let mut value_len_p = 0;
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let mut type_p: *const c_char = ptr::null_mut();
//
//        let err = api::add_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.value.as_ptr(), record.value.len(), record.tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), options_json.as_ptr(), &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record_id(wallet.handle, record_handle, &mut id_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), record.id);
//
//        let err = api::get_record_type(wallet.handle, record_handle, &mut type_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let err = api::get_record_value(wallet.handle, record_handle, &mut value_p, &mut value_len_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let err = api::free_record(wallet.handle, record_handle);
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_add_record_with_tags_then_fetch_default() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(true);
//
//        let options_json = CString::new(r#"{}"#).unwrap();
//        let mut record_handle = -1;
//        let mut id_p: *const c_char = ptr::null_mut();
//        let mut value_p: *const u8 = ptr::null_mut();
//        let mut value_len_p = 0;
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let mut type_p: *const c_char = ptr::null_mut();
//
//        let err = api::add_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.value.as_ptr(), record.value.len(), record.tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), options_json.as_ptr(), &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record_id(wallet.handle, record_handle, &mut id_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), record.id);
//
//        let err = api::get_record_type(wallet.handle, record_handle, &mut type_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let err = api::get_record_value(wallet.handle, record_handle, &mut value_p, &mut value_len_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { slice::from_raw_parts(value_p, value_len_p) }, record.value.as_slice());
//
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::Success);
//
//        let expected_tags_map: HashMap<String, serde_json::Value> = serde_json::from_slice(record.tags_json.as_bytes()).unwrap();
//        let tags_map: HashMap<String, serde_json::Value> = serde_json::from_str(unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap()).unwrap();
//        assert_eq!(tags_map, expected_tags_map);
//
//        let err = api::free_record(wallet.handle, record_handle);
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    /** Storage ADD_RECORD, GET_RECORD, DELETE_RECORD Tests WITHOUT TAGS */
//
//    #[test]
//    fn test_add_record_without_tags_then_fetch_all() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(true);
//
//        let options_json = fetch_options(true, true, true);
//        let mut record_handle = -1;
//        let mut id_p: *const c_char = ptr::null_mut();
//        let mut value_p: *const u8 = ptr::null_mut();
//        let mut value_len_p = 0;
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let mut type_p: *const c_char = ptr::null_mut();
//
//        let err = api::add_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.value.as_ptr(), record.value.len(), record.tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), options_json.as_ptr(), &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record_id(wallet.handle, record_handle, &mut id_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), record.id);
//
//        let err = api::get_record_type(wallet.handle, record_handle, &mut type_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(type_p) }.to_owned(), record.type_);
//
//        let err = api::get_record_value(wallet.handle, record_handle, &mut value_p, &mut value_len_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { slice::from_raw_parts(value_p, value_len_p) }, record.value.as_slice());
//
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::Success);
//
//        let expected_tags_map: HashMap<String, serde_json::Value> = serde_json::from_slice(record.tags_json.as_bytes()).unwrap();
//        let tags_map: HashMap<String, serde_json::Value> = serde_json::from_str(unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap()).unwrap();
//        assert_eq!(tags_map, expected_tags_map);
//
//        let err = api::free_record(wallet.handle, record_handle);
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_add_record_without_tags_then_fetch_only_type() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(false);
//
//        let options_json = fetch_options(true, false, false);
//        let mut record_handle = -1;
//        let mut id_p: *const c_char = ptr::null_mut();
//        let mut value_p: *const u8 = ptr::null_mut();
//        let mut value_len_p = 0;
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let mut type_p: *const c_char = ptr::null_mut();
//
//        let err = api::add_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.value.as_ptr(), record.value.len(), record.tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), options_json.as_ptr(), &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record_id(wallet.handle, record_handle, &mut id_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), record.id);
//
//        let err = api::get_record_type(wallet.handle, record_handle, &mut type_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(type_p) }.to_owned(), record.type_);
//
//        let err = api::get_record_value(wallet.handle, record_handle, &mut value_p, &mut value_len_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let err = api::free_record(wallet.handle, record_handle);
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_add_record_without_tags_then_fetch_only_value() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(false);
//
//        let options_json = fetch_options(false, true, false);
//        let mut record_handle = -1;
//        let mut id_p: *const c_char = ptr::null_mut();
//        let mut value_p: *const u8 = ptr::null_mut();
//        let mut value_len_p = 0;
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let mut type_p: *const c_char = ptr::null_mut();
//
//        let err = api::add_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.value.as_ptr(), record.value.len(), record.tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), options_json.as_ptr(), &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record_id(wallet.handle, record_handle, &mut id_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), record.id);
//
//        let err = api::get_record_type(wallet.handle, record_handle, &mut type_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let err = api::get_record_value(wallet.handle, record_handle, &mut value_p, &mut value_len_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { slice::from_raw_parts(value_p, value_len_p) }, record.value.as_slice());
//
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let err = api::free_record(wallet.handle, record_handle);
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_add_record_without_tags_then_fetch_only_tags() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(false);
//
//        let options_json = fetch_options(false, false, true);
//        let mut record_handle = -1;
//        let mut id_p: *const c_char = ptr::null_mut();
//        let mut value_p: *const u8 = ptr::null_mut();
//        let mut value_len_p = 0;
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let mut type_p: *const c_char = ptr::null_mut();
//
//        let err = api::add_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.value.as_ptr(), record.value.len(), record.tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), options_json.as_ptr(), &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record_id(wallet.handle, record_handle, &mut id_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), record.id);
//
//        let err = api::get_record_type(wallet.handle, record_handle, &mut type_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let err = api::get_record_value(wallet.handle, record_handle, &mut value_p, &mut value_len_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::Success);
//
//        let expected_tags_map: HashMap<String, serde_json::Value> = serde_json::from_slice(record.tags_json.as_bytes()).unwrap();
//        let tags_map: HashMap<String, serde_json::Value> = serde_json::from_str(unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap()).unwrap();
//        assert_eq!(tags_map, expected_tags_map);
//
//        let err = api::free_record(wallet.handle, record_handle);
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_add_record_without_tags_then_fetch_type_value() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(false);
//
//        let options_json = fetch_options(true, true, false);
//        let mut record_handle = -1;
//        let mut id_p: *const c_char = ptr::null_mut();
//        let mut value_p: *const u8 = ptr::null_mut();
//        let mut value_len_p = 0;
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let mut type_p: *const c_char = ptr::null_mut();
//
//        let err = api::add_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.value.as_ptr(), record.value.len(), record.tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), options_json.as_ptr(), &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record_id(wallet.handle, record_handle, &mut id_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), record.id);
//
//        let err = api::get_record_type(wallet.handle, record_handle, &mut type_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(type_p) }.to_owned(), record.type_);
//
//        let err = api::get_record_value(wallet.handle, record_handle, &mut value_p, &mut value_len_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { slice::from_raw_parts(value_p, value_len_p) }, record.value.as_slice());
//
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let err = api::free_record(wallet.handle, record_handle);
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_add_record_without_tags_then_fetch_type_tags() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(false);
//
//        let options_json = fetch_options(true, false, true);
//        let mut record_handle = -1;
//        let mut id_p: *const c_char = ptr::null_mut();
//        let mut value_p: *const u8 = ptr::null_mut();
//        let mut value_len_p = 0;
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let mut type_p: *const c_char = ptr::null_mut();
//
//        let err = api::add_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.value.as_ptr(), record.value.len(), record.tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), options_json.as_ptr(), &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record_id(wallet.handle, record_handle, &mut id_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), record.id);
//
//        let err = api::get_record_type(wallet.handle, record_handle, &mut type_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(type_p) }.to_owned(), record.type_);
//
//        let err = api::get_record_value(wallet.handle, record_handle, &mut value_p, &mut value_len_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::Success);
//
//        let expected_tags_map: HashMap<String, serde_json::Value> = serde_json::from_slice(record.tags_json.as_bytes()).unwrap();
//        let tags_map: HashMap<String, serde_json::Value> = serde_json::from_str(unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap()).unwrap();
//        assert_eq!(tags_map, expected_tags_map);
//
//        let err = api::free_record(wallet.handle, record_handle);
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_add_record_without_tags_then_fetch_value_tags() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(false);
//
//        let options_json = fetch_options(false, true, true);
//        let mut record_handle = -1;
//        let mut id_p: *const c_char = ptr::null_mut();
//        let mut value_p: *const u8 = ptr::null_mut();
//        let mut value_len_p = 0;
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let mut type_p: *const c_char = ptr::null_mut();
//
//       let err = api::add_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.value.as_ptr(), record.value.len(), record.tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), options_json.as_ptr(), &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record_id(wallet.handle, record_handle, &mut id_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), record.id);
//
//        let err = api::get_record_type(wallet.handle, record_handle, &mut type_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let err = api::get_record_value(wallet.handle, record_handle, &mut value_p, &mut value_len_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { slice::from_raw_parts(value_p, value_len_p) }, record.value.as_slice());
//
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::Success);
//
//        let expected_tags_map: HashMap<String, serde_json::Value> = serde_json::from_slice(record.tags_json.as_bytes()).unwrap();
//        let tags_map: HashMap<String, serde_json::Value> = serde_json::from_str(unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap()).unwrap();
//        assert_eq!(tags_map, expected_tags_map);
//
//        let err = api::free_record(wallet.handle, record_handle);
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_add_record_without_tags_then_fetch_none() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(false);
//
//        let options_json = fetch_options(false, false, false);
//        let mut record_handle = -1;
//        let mut id_p: *const c_char = ptr::null_mut();
//        let mut value_p: *const u8 = ptr::null_mut();
//        let mut value_len_p = 0;
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let mut type_p: *const c_char = ptr::null_mut();
//
//        let err = api::add_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.value.as_ptr(), record.value.len(), record.tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), options_json.as_ptr(), &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record_id(wallet.handle, record_handle, &mut id_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), record.id);
//
//        let err = api::get_record_type(wallet.handle, record_handle, &mut type_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let err = api::get_record_value(wallet.handle, record_handle, &mut value_p, &mut value_len_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let err = api::free_record(wallet.handle, record_handle);
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_add_record_without_tags_then_fetch_default() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(false);
//
//        let options_json = CString::new(r#"{}"#).unwrap();
//        let mut record_handle = -1;
//        let mut id_p: *const c_char = ptr::null_mut();
//        let mut value_p: *const u8 = ptr::null_mut();
//        let mut value_len_p = 0;
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let mut type_p: *const c_char = ptr::null_mut();
//
//        let err = api::add_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.value.as_ptr(), record.value.len(), record.tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), options_json.as_ptr(), &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record_id(wallet.handle, record_handle, &mut id_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), record.id);
//
//        let err = api::get_record_type(wallet.handle, record_handle, &mut type_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let err = api::get_record_value(wallet.handle, record_handle, &mut value_p, &mut value_len_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { slice::from_raw_parts(value_p, value_len_p) }, record.value.as_slice());
//
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::Success);
//
//        let expected_tags_map: HashMap<String, serde_json::Value> = serde_json::from_slice(record.tags_json.as_bytes()).unwrap();
//        let tags_map: HashMap<String, serde_json::Value> = serde_json::from_str(unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap()).unwrap();
//        assert_eq!(tags_map, expected_tags_map);
//
//        let err = api::free_record(wallet.handle, record_handle);
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_duplicate_add_record_without_tags_then_fetch_default() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(false);
//
//        let options_json = CString::new(r#"{}"#).unwrap();
//        let mut record_handle = -1;
//        let mut id_p: *const c_char = ptr::null_mut();
//        let mut value_p: *const u8 = ptr::null_mut();
//        let mut value_len_p = 0;
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let mut type_p: *const c_char = ptr::null_mut();
//
//        let err = api::add_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.value.as_ptr(), record.value.len(), record.tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), options_json.as_ptr(), &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record_id(wallet.handle, record_handle, &mut id_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), record.id);
//
//        let err = api::get_record_type(wallet.handle, record_handle, &mut type_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let err = api::get_record_value(wallet.handle, record_handle, &mut value_p, &mut value_len_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { slice::from_raw_parts(value_p, value_len_p) }, record.value.as_slice());
//
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::Success);
//
//        let expected_tags_map: HashMap<String, serde_json::Value> = serde_json::from_slice(record.tags_json.as_bytes()).unwrap();
//        let tags_map: HashMap<String, serde_json::Value> = serde_json::from_str(unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap()).unwrap();
//        assert_eq!(tags_map, expected_tags_map);
//
//        let err = api::free_record(wallet.handle, record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::add_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.value.as_ptr(), record.value.len(), record.tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::ItemAlreadyExists);
//    }
//
//    #[test]
//    fn test_add_record_invalid_handle() {
//        let wallet = TestWallet::new_default(true);
//
//        let record = TestRecord::new_default(false);
//
//        let err = api::add_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.value.as_ptr(), record.value.len(), record.tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::InvalidState);
//    }
//
//    #[test]
//    fn test_add_record_with_large_type_value() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(false);
//
//        let large_type = CString::new(random_string(100)).unwrap();
//
//        let err = api::add_record(wallet.handle, large_type.as_ptr(), record.id.as_ptr(), record.value.as_ptr(), record.value.len(), record.tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::IOError);
//    }
//
//    #[test]
//    fn test_add_record_with_tags_invalid_json_format() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(false);
//
//        let invalid_tags_json = CString::new("...").unwrap();
//
//        let err = api::add_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.value.as_ptr(), record.value.len(), invalid_tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_add_record_with_tags_invalid_json_format_1() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(false);
//
//        let invalid_tags_json = CString::new(r#""'''{"key": "value"}"#).unwrap();
//
//        let err = api::add_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.value.as_ptr(), record.value.len(), invalid_tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_get_record_invalid_options_format() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(false);
//
//        wallet.add_record(&record);
//
//        let options_json = CString::new("//").unwrap();
//        let mut record_handle = -1;
//
//        let err = api::get_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), options_json.as_ptr(), &mut record_handle);
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_get_record_record_not_exists() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(false);
//
//        let mut record_handle = -1;
//
//        let options_json = fetch_options(false, false, false);
//
//        let err = api::get_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), options_json.as_ptr(), &mut record_handle);
//        assert_eq!(err, ErrorCode::ItemNotFound);
//    }
//
//    #[test]
//    fn test_delete_record() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(false);
//
//        wallet.add_record(&record);
//
//        let err = api::delete_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_delete_unknown_record() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(false);
//
//        let err = api::delete_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr());
//        assert_eq!(err, ErrorCode::ItemNotFound);
//    }
//
//    #[test]
//    fn test_delete_record_invalid_handle() {
//        let wallet = TestWallet::new_default(true);
//        let record = TestRecord::new_default(false);
//
//        let err = api::delete_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr());
//        assert_eq!(err, ErrorCode::InvalidState);
//    }
//
//    #[test]
//    fn test_free_record_invalid_storage_handle() {
//        let wallet = TestWallet::new_default(true);
//
//        let record_handle = -1;
//
//        let err = api::free_record(wallet.handle, record_handle);
//        assert_eq!(err, ErrorCode::InvalidState);
//    }
//
//    #[test]
//    fn test_free_record_invalid_record_handle() {
//        let wallet = TestWallet::new_default(false);
//
//        let record_handle = -1;
//
//        let err = api::free_record(wallet.handle, record_handle);
//        assert_eq!(err, ErrorCode::InvalidState);
//    }
//
//    /** Storage UPDATE_RECORD_VALUE Tests */
//
//    #[test]
//    fn test_update_record_value() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(true);
//
//        wallet.add_record(&record);
//
//        let new_value = vec![2, 5, 8, 13];
//        let err = api::update_record_value(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), new_value.as_ptr(), new_value.len());
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut record_handle = -1;
//        let options_json = fetch_options(false, true, false);
//        let err = api::get_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), options_json.as_ptr(), &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut value_p: *const u8 = ptr::null_mut();
//        let mut value_len_p = 0;
//        let err = api::get_record_value(wallet.handle, record_handle, &mut value_p, &mut value_len_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { slice::from_raw_parts(value_p, value_len_p) }, new_value.as_slice());
//    }
//
//    #[test]
//    fn test_update_record_value_with_same_value() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(true);
//
//        wallet.add_record(&record);
//
//        let new_value = vec![1, 2, 3, 4];
//        let err = api::update_record_value(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), new_value.as_ptr(), new_value.len());
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut record_handle = -1;
//        let options_json = fetch_options(false, true, false);
//        let err = api::get_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), options_json.as_ptr(), &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut value_p: *const u8 = ptr::null_mut();
//        let mut value_len_p = 0;
//        let err = api::get_record_value(wallet.handle, record_handle, &mut value_p, &mut value_len_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { slice::from_raw_parts(value_p, value_len_p) }, new_value.as_slice());
//    }
//
//    #[test]
//    fn test_update_value_invalid_handle() {
//        let wallet = TestWallet::new_default(true);
//        let record = TestRecord::new_default(true);
//
//        let err = api::update_record_value(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.value.as_ptr(), record.value.len());
//        assert_eq!(err, ErrorCode::InvalidState);
//    }
//
//    #[test]
//    fn test_update_value_for_unknown_record() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(true);
//
//        let err = api::update_record_value(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.value.as_ptr(), record.value.len());
//        assert_eq!(err, ErrorCode::ItemNotFound);
//    }
//
//    /** Storage ADD_RECORD_TAGS Tests */
//
//    #[test]
//    fn test_add_record_tags_record_exists_with_tags() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(false);
//
//        wallet.add_record(&record);
//
//        let tags_json = CString::new(r#"{"tag1": "value1", "tag2": "value2", "~tag3": "value3"}"#).unwrap();
//        let err = api::add_record_tags(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut record_handle = -1;
//        let options_json = fetch_options(false, true, true);
//        let err = api::get_record(wallet.handle, record.type_.as_ptr(),record.id.as_ptr(), options_json.as_ptr(), &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::Success);
//
//        let expected_tags_map: HashMap<String, serde_json::Value> = serde_json::from_slice(tags_json.as_bytes()).unwrap();
//        let tags_map: HashMap<String, serde_json::Value> = serde_json::from_str(unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap()).unwrap();
//        assert_eq!(tags_map, expected_tags_map);
//    }
//
//    #[test]
//    fn test_add_record_tags_record_exists_without_tags() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(true);
//
//        wallet.add_record(&record);
//
//        let tags_json = CString::new(r#"{"tag1": "value1", "tag2": "value2", "~tag3": "value3"}"#).unwrap();
//        let err = api::add_record_tags(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut record_handle = -1;
//        let options_json = fetch_options(false, true, true);
//        let err = api::get_record(wallet.handle, record.type_.as_ptr(),record.id.as_ptr(), options_json.as_ptr(), &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::Success);
//
//        let expected_tags_map: HashMap<String, serde_json::Value> = serde_json::from_slice(tags_json.as_bytes()).unwrap();
//        let tags_map: HashMap<String, serde_json::Value> = serde_json::from_str(unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap()).unwrap();
//        assert_eq!(tags_map, expected_tags_map);
//    }
//
//    #[test]
//    fn test_add_empty_record_tags_record_with_tags() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(true);
//
//        wallet.add_record(&record);
//
//        let err = api::add_record_tags(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut record_handle = -1;
//        let options_json = fetch_options(false, true, true);
//        let err = api::get_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), options_json.as_ptr(), &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::Success);
//
//        let expected_tags_map: HashMap<String, serde_json::Value> = serde_json::from_slice(record.tags_json.as_bytes()).unwrap();
//        let tags_map: HashMap<String, serde_json::Value> = serde_json::from_str(unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap()).unwrap();
//        assert_eq!(tags_map, expected_tags_map);
//    }
//
//    #[test]
//    fn test_add_empty_record_tags_record_without_tags() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(false);
//
//        wallet.add_record(&record);
//
//        let err = api::add_record_tags(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut record_handle = -1;
//        let options_json = fetch_options(false, true, true);
//        let err = api::get_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), options_json.as_ptr(), &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::Success);
//
//        let expected_tags_map: HashMap<String, serde_json::Value> = serde_json::from_slice(record.tags_json.as_bytes()).unwrap();
//        let tags_map: HashMap<String, serde_json::Value> = serde_json::from_str(unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap()).unwrap();
//        assert_eq!(tags_map, expected_tags_map);
//    }
//
//    #[test]
//    fn test_add_duplicate_record_tags() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(true);
//
//        wallet.add_record(&record);
//
//        let err = api::add_record_tags(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::add_record_tags(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut record_handle = -1;
//        let options_json = fetch_options(false, true, true);
//        let err = api::get_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), options_json.as_ptr(), &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::Success);
//
//        let expected_tags_map: HashMap<String, serde_json::Value> = serde_json::from_slice(record.tags_json.as_bytes()).unwrap();
//        let tags_map: HashMap<String, serde_json::Value> = serde_json::from_str(unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap()).unwrap();
//        assert_eq!(tags_map, expected_tags_map);
//    }
//
//    #[test]
//    fn test_add_record_tags_unknown_record() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(false);
//
//        let err = api::add_record_tags(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::ItemNotFound);
//    }
//
//    #[test]
//    fn test_add_record_tags_invalid_format() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(false);
//
//        let tags_json = CString::new(r#"{"'''tag1": "value1", "tag2": "value2", "~tag3": "value3"}"#).unwrap();
//        let err = api::add_record_tags(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_add_tags_invalid_handle() {
//        let wallet = TestWallet::new_default(true);
//        let record = TestRecord::new_default(true);
//
//        let err = api::add_record_tags(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::InvalidState);
//    }
//
//    #[test]
//    fn test_add_tags_for_unknown_record() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(true);
//
//        let err = api::add_record_tags(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::ItemNotFound);
//    }
//
//    /** Storage UPDATE_RECORD_TAGS Tests */
//
//    #[test]
//    fn test_update_tags_record_with_tags() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(true);
//
//
//        wallet.add_record(&record);
//
//        let new_tags_json = CString::new(r#"{"tag1": "value1_new", "tag2": "value2_new", "~tag3": "value3_new"}"#).unwrap();
//        let err = api::update_record_tags(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), new_tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut record_handle = -1;
//        let options_json = fetch_options(false, true, true);
//        let err = api::get_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), options_json.as_ptr(), &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::Success);
//
//        let expected_tags_map: HashMap<String, serde_json::Value> = serde_json::from_slice(new_tags_json.as_bytes()).unwrap();
//        let tags_map: HashMap<String, serde_json::Value> = serde_json::from_str(unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap()).unwrap();
//        assert_eq!(tags_map, expected_tags_map);
//    }
//
//    #[test]
//    fn test_update_tags_record_without_tags() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(false);
//
//        wallet.add_record(&record);
//
//        let new_tags_json = CString::new(r#"{"tag1": "value1_new", "tag2": "value2_new", "~tag3": "value3_new"}"#).unwrap();
//        let err = api::update_record_tags(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), new_tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut record_handle = -1;
//        let options_json = fetch_options(false, true, true);
//        let err = api::get_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), options_json.as_ptr(), &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::Success);
//
//        let expected_tags_map: HashMap<String, serde_json::Value> = serde_json::from_slice(new_tags_json.as_bytes()).unwrap();
//        let tags_map: HashMap<String, serde_json::Value> = serde_json::from_str(unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap()).unwrap();
//        assert_eq!(tags_map, expected_tags_map);
//    }
//
//    #[test]
//    fn test_update_record_tags_same_value() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(false);
//
//        wallet.add_record(&record);
//
//        let err = api::update_record_tags(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_update_record_tags_invalid_format() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(false);
//
//        let invalid_tags = CString::new("...").unwrap();
//        let err = api::update_record_tags(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), invalid_tags.as_ptr());
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_update_record_tag_unknown_record() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(false);
//
//        let err = api::update_record_tags(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::ItemNotFound);
//    }
//
//    #[test]
//    fn test_update_record_tag_invalid_storage_handle() {
//        let wallet = TestWallet::new_default(true);
//        let record = TestRecord::new_default(false);
//
//        let err = api::update_record_tags(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::InvalidState);
//    }
//
//    /** Storage DELETE_RECORD_TAGS Tests */
//
//    #[test]
//    fn test_delete_record_tags() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(false);
//
//        wallet.add_record(&record);
//
//        let err = api::delete_record_tags(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.tag_names.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut record_handle = -1;
//        let options_json = fetch_options(false, false, true);
//        let err = api::get_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), options_json.as_ptr(), &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::Success);
//
//        let tags_json_empty = CString::new(r#"{}"#).unwrap();
//        let expected_tags_map: HashMap<String, serde_json::Value> = serde_json::from_slice(tags_json_empty.as_bytes()).unwrap();
//        let tags_map: HashMap<String, serde_json::Value> = serde_json::from_str(unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap()).unwrap();
//        assert_eq!(tags_map, expected_tags_map);
//    }
//
//    #[test]
//    fn test_delete_unknown_tags() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(true);
//
//        wallet.add_record(&record);
//
//        let unknown_tags = CString::new(r#"["bogus1", "bogus2", "~bogus3"]"#).unwrap();
//        let err = api::delete_record_tags(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), unknown_tags.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut record_handle = -1;
//        let options_json = fetch_options(false, false, true);
//        let err = api::get_record(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), options_json.as_ptr(), &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::Success);
//
//        let expected_tags_map: HashMap<String, serde_json::Value> = serde_json::from_slice(record.tags_json.as_bytes()).unwrap();
//        let tags_map: HashMap<String, serde_json::Value> = serde_json::from_str(unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap()).unwrap();
//        assert_eq!(tags_map, expected_tags_map);
//    }
//
//    #[test]
//    fn test_delete_tags_invalid_json_format() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(true);
//
//        wallet.add_record(&record);
//
//        let tag_names = CString::new(r#"["'tag11", "tag22", "~tag33"]"#).unwrap();
//        let err = api::delete_record_tags(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), tag_names.as_ptr());
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_delete_tags_empty_tag_name_list() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(false);
//
//        wallet.add_record(&record);
//
//        let err = api::delete_record_tags(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.tag_names.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_delete_tags_unknown_record() {
//        let wallet = TestWallet::new_default(false);
//        let record = TestRecord::new_default(false);
//
//        let err = api::delete_record_tags(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.tag_names.as_ptr());
//        assert_eq!(err, ErrorCode::ItemNotFound);
//    }
//
//    #[test]
//    fn test_delete_unknown_tags_invalid_storage_handle() {
//        let wallet = TestWallet::new_default(true);
//        let record = TestRecord::new_default(false);
//
//        let err = api::delete_record_tags(wallet.handle, record.type_.as_ptr(), record.id.as_ptr(), record.tag_names.as_ptr());
//        assert_eq!(err, ErrorCode::InvalidState);
//    }
//
//    /** Storage METADATA Tests */
//
//    #[test]
//    fn test_set_get_metadata() {
//        let wallet = TestWallet::new_default(false);
//
//        let mut metadata_handle = -1;
//        let mut metadata_ptr: *const c_char = ptr::null_mut();
//
//        let new_metadata = random_string(512);
//        let new_metadata_cstring = CString::new(new_metadata.clone()).unwrap();
//
//        let err = api::set_metadata(wallet.handle, new_metadata_cstring.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_metadata(wallet.handle, &mut metadata_ptr, &mut metadata_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let metadata = unsafe { CStr::from_ptr(metadata_ptr).to_str().unwrap() };
//
//        assert_eq!(new_metadata, metadata);
//
//        let err = api::free_metadata(wallet.handle, metadata_handle);
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_set_metadata_invalid_metadata_handle() {
//        let wallet = TestWallet::new_default(true);
//
//        let new_metadata = random_string(1);
//        let new_metadata_cstring = CString::new(new_metadata.clone()).unwrap();
//
//        let err = api::set_metadata(wallet.handle, new_metadata_cstring.as_ptr());
//        assert_eq!(err, ErrorCode::InvalidState);
//    }
//
//    #[test]
//    fn test_free_metadata_invalid_storage_handle() {
//        let wallet = TestWallet::new_default(true);
//
//        let metadata_handle = -1;
//
//        let err = api::free_metadata(wallet.handle, metadata_handle);
//        assert_eq!(err, ErrorCode::InvalidState);
//    }
//
//    #[test]
//    fn test_free_metadata_invalid_metadata_handle() {
//        let wallet = TestWallet::new_default(false);
//
//        let metadata_handle = -1;
//
//        let err = api::free_metadata(wallet.handle, metadata_handle);
//        assert_eq!(err, ErrorCode::InvalidState);
//    }
//
//    /** Search Record Tests */
//
//    #[test]
//    fn test_search_records_fetch_all() {
//        let wallet = TestWallet::new_default(false);
//
//        let type_ = CString::new(random_string(10)).unwrap();
//
//        // -- Add records --
//        let id_1 = CString::new(random_name()).unwrap();
//        let value_1 = vec![1, 2, 3, 4];
//        let tags_json_1 = CString::new(r#"{"tag1": "value_nem1", "tag2": "value_nem2", "~tag3": "value_nem3"}"#).unwrap();
//
//        let err = api::add_record(wallet.handle, type_.as_ptr(), id_1.as_ptr(), value_1.as_ptr(), value_1.len(), tags_json_1.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let id_2 = CString::new(random_name()).unwrap();
//        let value_2 = vec![1, 2, 3, 4];
//        let tags_json_2 = CString::new(r#"{"tag1": "value_nem11", "tag2": "value_nem22", "~tag3": "value_nem33"}"#).unwrap();
//
//        let err = api::add_record(wallet.handle, type_.as_ptr(), id_2.as_ptr(), value_2.as_ptr(), value_2.len(), tags_json_2.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        // -- Search Records --
//        let query_json = json!({
//            "tag1": {"$in": ["value_nem1", "value_nem11"]},
//            "~tag3": {"$in": ["value_nem3", "value_nem33"]},
//            "$not": {
//                "tag2": "value_nem22"
//            }
//        });
//
//        let query_json = serde_json::to_string(&query_json).unwrap();
//        let query_json = CString::new(query_json).unwrap();
//
//        let options_json = search_options(true, true, true, true, true);
//
//        let mut search_handle: i32 = -1;
//
//        let err = api::search_records(wallet.handle, type_.as_ptr(), query_json.as_ptr(), options_json.as_ptr(), &mut search_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut total_count = 0;
//        let err = api::get_search_total_count(wallet.handle, search_handle, &mut total_count);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(total_count, 1);
//
//        let mut record_handle = -1;
//        let err = api::fetch_search_next_record(wallet.handle, search_handle, &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        // Check type
//        let mut type_p: *const c_char = ptr::null_mut();
//        let err = api::get_record_type(wallet.handle, record_handle, &mut type_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(type_p) }.to_owned(), type_);
//
//        // Check id/name
//        let mut id_p: *const c_char = ptr::null_mut();
//        let err = api::get_record_id(wallet.handle, record_handle, &mut id_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), id_1);
//
//        // Check value
//        let mut value_p: *const u8 = ptr::null_mut();
//        let mut value_len_p = 0;
//        let err = api::get_record_value(wallet.handle, record_handle, &mut value_p, &mut value_len_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { slice::from_raw_parts(value_p, value_len_p) }, value_1.as_slice());
//
//        // Check tags
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::Success);
//
//        let expected_tags_map: HashMap<String, serde_json::Value> = serde_json::from_slice(tags_json_1.as_bytes()).unwrap();
//        let tags_map: HashMap<String, serde_json::Value> = serde_json::from_str(unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap()).unwrap();
//        assert_eq!(tags_map, expected_tags_map);
//
//        // No more records in the result set
//        let err = api::fetch_search_next_record(wallet.handle, search_handle, &mut record_handle);
//        assert_eq!(err, ErrorCode::ItemNotFound);
//
//        // After the iterator is exhausted search handle is invalidated
//        let err = api::fetch_search_next_record(wallet.handle, search_handle, &mut record_handle);
//        assert_eq!(err, ErrorCode::ItemNotFound);
//
//        let err = api::free_search(wallet.handle, search_handle);
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_search_records_fetch_all_empty_query_string() {
//        let wallet = TestWallet::new_default(false);
//
//        let type_ = CString::new(random_string(10)).unwrap();
//
//        // -- Add records --
//        let id_1 = CString::new(random_name()).unwrap();
//        let value_1 = vec![1, 2, 3, 4];
//        let tags_json_1 = CString::new(r#"{"tag1": "value_nem1", "tag2": "value_nem2", "~tag3": "value_nem3"}"#).unwrap();
//
//        let err = api::add_record(wallet.handle, type_.as_ptr(), id_1.as_ptr(), value_1.as_ptr(), value_1.len(), tags_json_1.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let id_2 = CString::new(random_name()).unwrap();
//        let value_2 = vec![1, 2, 3, 4];
//        let tags_json_2 = CString::new(r#"{"tag1": "value_nem11", "tag2": "value_nem22", "~tag3": "value_nem33"}"#).unwrap();
//
//        let err = api::add_record(wallet.handle, type_.as_ptr(), id_2.as_ptr(), value_2.as_ptr(), value_2.len(), tags_json_2.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        // -- Search Records --
//        let query_json = CString::new(r#"{}"#).unwrap();
//
//        let options_json = search_options(true, true, true, true, true);
//
//        let mut search_handle: i32 = -1;
//
//        let err = api::search_records(wallet.handle, type_.as_ptr(), query_json.as_ptr(), options_json.as_ptr(), &mut search_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut total_count = 0;
//        let err = api::get_search_total_count(wallet.handle, search_handle, &mut total_count);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(total_count, 2);
//
//        // Record 1
//        let mut record_handle = -1;
//        let err = api::fetch_search_next_record(wallet.handle, search_handle, &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        // Record 2
//
//        let err = api::fetch_search_next_record(wallet.handle, search_handle, &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        // No more records in the result set
//        let err = api::fetch_search_next_record(wallet.handle, search_handle, &mut record_handle);
//        assert_eq!(err, ErrorCode::ItemNotFound);
//
//        let err = api::free_search(wallet.handle, search_handle);
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_search_records_fetch_type_only() {
//        let wallet = TestWallet::new_default(false);
//
//        let type_ = CString::new(random_string(10)).unwrap();
//
//        // -- Add records --
//        let id_1 = CString::new(random_name()).unwrap();
//        let value_1 = vec![1, 2, 3, 4];
//        let tags_json_1 = CString::new(r#"{"tag1": "value_nem1", "tag2": "value_nem2", "~tag3": "value_nem3"}"#).unwrap();
//
//        let err = api::add_record(wallet.handle, type_.as_ptr(), id_1.as_ptr(), value_1.as_ptr(), value_1.len(), tags_json_1.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let id_2 = CString::new(random_name()).unwrap();
//        let value_2 = vec![1, 2, 3, 4];
//        let tags_json_2 = CString::new(r#"{"tag1": "value_nem11", "tag2": "value_nem22", "~tag3": "value_nem33"}"#).unwrap();
//
//        let err = api::add_record(wallet.handle, type_.as_ptr(), id_2.as_ptr(), value_2.as_ptr(), value_2.len(), tags_json_2.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        // -- Search Records --
//        let query_json = json!({
//            "tag1": {"$in": ["value_nem1", "value_nem11"]},
//            "~tag3": {"$in": ["value_nem3", "value_nem33"]},
//            "$not": {
//                "tag2": "value_nem22"
//            }
//        });
//
//        let query_json = serde_json::to_string(&query_json).unwrap();
//        let query_json = CString::new(query_json).unwrap();
//
//        let options_json = search_options(true, false, true, false, false);
//
//        let mut search_handle: i32 = -1;
//
//        let err = api::search_records(wallet.handle, type_.as_ptr(), query_json.as_ptr(), options_json.as_ptr(), &mut search_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut total_count = 0;
//        let err = api::get_search_total_count(wallet.handle, search_handle, &mut total_count);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let mut record_handle = -1;
//        let err = api::fetch_search_next_record(wallet.handle, search_handle, &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        // Check type
//        let mut type_p: *const c_char = ptr::null_mut();
//        let err = api::get_record_type(wallet.handle, record_handle, &mut type_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(type_p) }.to_owned(), type_);
//
//        // Check id/name
//        let mut id_p: *const c_char = ptr::null_mut();
//        let err = api::get_record_id(wallet.handle, record_handle, &mut id_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), id_1);
//
//        // Check value
//        let mut value_p: *const u8 = ptr::null_mut();
//        let mut value_len_p = 0;
//        let err = api::get_record_value(wallet.handle, record_handle, &mut value_p, &mut value_len_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        // Check tags
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let err = api::free_search(wallet.handle, search_handle);
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_search_records_fetch_value_only() {
//        let wallet = TestWallet::new_default(false);
//
//        let type_ = CString::new(random_string(10)).unwrap();
//
//        // -- Add records --
//        let id_1 = CString::new(random_name()).unwrap();
//        let value_1 = vec![1, 2, 3, 4];
//        let tags_json_1 = CString::new(r#"{"tag1": "value_nem1", "tag2": "value_nem2", "~tag3": "value_nem3"}"#).unwrap();
//
//        let err = api::add_record(wallet.handle, type_.as_ptr(), id_1.as_ptr(), value_1.as_ptr(), value_1.len(), tags_json_1.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let id_2 = CString::new(random_name()).unwrap();
//        let value_2 = vec![1, 2, 3, 4];
//        let tags_json_2 = CString::new(r#"{"tag1": "value_nem11", "tag2": "value_nem22", "~tag3": "value_nem33"}"#).unwrap();
//
//        let err = api::add_record(wallet.handle, type_.as_ptr(), id_2.as_ptr(), value_2.as_ptr(), value_2.len(), tags_json_2.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        // -- Search Records --
//        let query_json = json!({
//            "tag1": {"$in": ["value_nem1", "value_nem11"]},
//            "~tag3": {"$in": ["value_nem3", "value_nem33"]},
//            "$not": {
//                "tag2": "value_nem22"
//            }
//        });
//
//        let query_json = serde_json::to_string(&query_json).unwrap();
//        let query_json = CString::new(query_json).unwrap();
//
//        let options_json = search_options(true, false, false, true, false);
//
//        let mut search_handle: i32 = -1;
//
//        let err = api::search_records(wallet.handle, type_.as_ptr(), query_json.as_ptr(), options_json.as_ptr(), &mut search_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut total_count = 0;
//        let err = api::get_search_total_count(wallet.handle, search_handle, &mut total_count);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let mut record_handle = -1;
//        let err = api::fetch_search_next_record(wallet.handle, search_handle, &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        // Check type
//        let mut type_p: *const c_char = ptr::null_mut();
//        let err = api::get_record_type(wallet.handle, record_handle, &mut type_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        // Check id/name
//        let mut id_p: *const c_char = ptr::null_mut();
//        let err = api::get_record_id(wallet.handle, record_handle, &mut id_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), id_1);
//
//        // Check value
//        let mut value_p: *const u8 = ptr::null_mut();
//        let mut value_len_p = 0;
//        let err = api::get_record_value(wallet.handle, record_handle, &mut value_p, &mut value_len_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { slice::from_raw_parts(value_p, value_len_p) }, value_1.as_slice());
//
//        // Check tags
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let err = api::free_search(wallet.handle, search_handle);
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_search_records_fetch_tags_only() {
//        let wallet = TestWallet::new_default(false);
//
//        let type_ = CString::new(random_string(10)).unwrap();
//
//        // -- Add records --
//        let id_1 = CString::new(random_name()).unwrap();
//        let value_1 = vec![1, 2, 3, 4];
//        let tags_json_1 = CString::new(r#"{"tag1": "value_nem1", "tag2": "value_nem2", "~tag3": "value_nem3"}"#).unwrap();
//
//        let err = api::add_record(wallet.handle, type_.as_ptr(), id_1.as_ptr(), value_1.as_ptr(), value_1.len(), tags_json_1.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let id_2 = CString::new(random_name()).unwrap();
//        let value_2 = vec![1, 2, 3, 4];
//        let tags_json_2 = CString::new(r#"{"tag1": "value_nem11", "tag2": "value_nem22", "~tag3": "value_nem33"}"#).unwrap();
//
//        let err = api::add_record(wallet.handle, type_.as_ptr(), id_2.as_ptr(), value_2.as_ptr(), value_2.len(), tags_json_2.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        // -- Search Records --
//        let query_json = json!({
//            "tag1": {"$in": ["value_nem1", "value_nem11"]},
//            "~tag3": {"$in": ["value_nem3", "value_nem33"]},
//            "$not": {
//                "tag2": "value_nem22"
//            }
//        });
//
//        let query_json = serde_json::to_string(&query_json).unwrap();
//        let query_json = CString::new(query_json).unwrap();
//
//        let options_json = search_options(true, false, false, false, true);
//
//        let mut search_handle: i32 = -1;
//
//        let err = api::search_records(wallet.handle, type_.as_ptr(), query_json.as_ptr(), options_json.as_ptr(), &mut search_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut total_count = 0;
//        let err = api::get_search_total_count(wallet.handle, search_handle, &mut total_count);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let mut record_handle = -1;
//        let err = api::fetch_search_next_record(wallet.handle, search_handle, &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        // Check type
//        let mut type_p: *const c_char = ptr::null_mut();
//        let err = api::get_record_type(wallet.handle, record_handle, &mut type_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        // Check id/name
//        let mut id_p: *const c_char = ptr::null_mut();
//        let err = api::get_record_id(wallet.handle, record_handle, &mut id_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), id_1);
//
//        // Check value
//        let mut value_p: *const u8 = ptr::null_mut();
//        let mut value_len_p = 0;
//        let err = api::get_record_value(wallet.handle, record_handle, &mut value_p, &mut value_len_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        // Check tags
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::Success);
//
//        let expected_tags_map: HashMap<String, serde_json::Value> = serde_json::from_slice(tags_json_1.as_bytes()).unwrap();
//        let tags_map: HashMap<String, serde_json::Value> = serde_json::from_str(unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap()).unwrap();
//        assert_eq!(tags_map, expected_tags_map);
//
//        let err = api::free_search(wallet.handle, search_handle);
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_search_records_fetch_records_only() {
//        let wallet = TestWallet::new_default(false);
//
//        let type_ = CString::new(random_string(10)).unwrap();
//
//        // -- Add records --
//        let id_1 = CString::new(random_name()).unwrap();
//        let value_1 = vec![1, 2, 3, 4];
//        let tags_json_1 = CString::new(r#"{"tag1": "value_nem1", "tag2": "value_nem2", "~tag3": "value_nem3"}"#).unwrap();
//
//        let err = api::add_record(wallet.handle, type_.as_ptr(), id_1.as_ptr(), value_1.as_ptr(), value_1.len(), tags_json_1.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let id_2 = CString::new(random_name()).unwrap();
//        let value_2 = vec![1, 2, 3, 4];
//        let tags_json_2 = CString::new(r#"{"tag1": "value_nem11", "tag2": "value_nem22", "~tag3": "value_nem33"}"#).unwrap();
//
//        let err = api::add_record(wallet.handle, type_.as_ptr(), id_2.as_ptr(), value_2.as_ptr(), value_2.len(), tags_json_2.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        // -- Search Records --
//        let query_json = json!({
//            "tag1": {"$in": ["value_nem1", "value_nem11"]},
//            "~tag3": {"$in": ["value_nem3", "value_nem33"]},
//            "$not": {
//                "tag2": "value_nem22"
//            }
//        });
//
//        let query_json = serde_json::to_string(&query_json).unwrap();
//        let query_json = CString::new(query_json).unwrap();
//
//        let options_json = search_options(true, false, false, false, false);
//
//        let mut search_handle: i32 = -1;
//
//        let err = api::search_records(wallet.handle, type_.as_ptr(), query_json.as_ptr(), options_json.as_ptr(), &mut search_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut total_count = 0;
//        let err = api::get_search_total_count(wallet.handle, search_handle, &mut total_count);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let mut record_handle = -1;
//        let err = api::fetch_search_next_record(wallet.handle, search_handle, &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        // Check type
//        let mut type_p: *const c_char = ptr::null_mut();
//        let err = api::get_record_type(wallet.handle, record_handle, &mut type_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        // Check id/name
//        let mut id_p: *const c_char = ptr::null_mut();
//        let err = api::get_record_id(wallet.handle, record_handle, &mut id_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), id_1);
//
//        // Check value
//        let mut value_p: *const u8 = ptr::null_mut();
//        let mut value_len_p = 0;
//        let err = api::get_record_value(wallet.handle, record_handle, &mut value_p, &mut value_len_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        // Check tags
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//    }
//
//    #[test]
//    fn test_search_records_fetch_count_only() {
//        let wallet = TestWallet::new_default(false);
//
//        let type_ = CString::new(random_string(10)).unwrap();
//
//        // -- Add records --
//        let id_1 = CString::new(random_name()).unwrap();
//        let value_1 = vec![1, 2, 3, 4];
//        let tags_json_1 = CString::new(r#"{"tag1": "value_nem1", "tag2": "value_nem2", "~tag3": "value_nem3"}"#).unwrap();
//
//        let err = api::add_record(wallet.handle, type_.as_ptr(), id_1.as_ptr(), value_1.as_ptr(), value_1.len(), tags_json_1.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let id_2 = CString::new(random_name()).unwrap();
//        let value_2 = vec![1, 2, 3, 4];
//        let tags_json_2 = CString::new(r#"{"tag1": "value_nem11", "tag2": "value_nem22", "~tag3": "value_nem33"}"#).unwrap();
//
//        let err = api::add_record(wallet.handle, type_.as_ptr(), id_2.as_ptr(), value_2.as_ptr(), value_2.len(), tags_json_2.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        // -- Search Records --
//        let query_json = json!({
//            "tag1": {"$in": ["value_nem1", "value_nem11"]},
//            "~tag3": {"$in": ["value_nem3", "value_nem33"]},
//            "$not": {
//                "tag2": "value_nem22"
//            }
//        });
//
//        let query_json = serde_json::to_string(&query_json).unwrap();
//        let query_json = CString::new(query_json).unwrap();
//
//        let options_json = search_options(false, true, false, false, false);
//
//        let mut search_handle: i32 = -1;
//
//        let err = api::search_records(wallet.handle, type_.as_ptr(), query_json.as_ptr(), options_json.as_ptr(), &mut search_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut total_count = 0;
//        let err = api::get_search_total_count(wallet.handle, search_handle, &mut total_count);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(total_count, 1);
//
//        let mut record_handle = -1;
//        let err = api::fetch_search_next_record(wallet.handle, search_handle, &mut record_handle);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let err = api::free_search(wallet.handle, search_handle);
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_search_records_fetch_default() {
//        let wallet = TestWallet::new_default(false);
//
//        let type_ = CString::new(random_string(10)).unwrap();
//
//        // -- Add records --
//        let id_1 = CString::new(random_name()).unwrap();
//        let value_1 = vec![1, 2, 3, 4];
//        let tags_json_1 = CString::new(r#"{"tag1": "value_nem1", "tag2": "value_nem2", "~tag3": "value_nem3"}"#).unwrap();
//
//        let err = api::add_record(wallet.handle, type_.as_ptr(), id_1.as_ptr(), value_1.as_ptr(), value_1.len(), tags_json_1.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let id_2 = CString::new(random_name()).unwrap();
//        let value_2 = vec![1, 2, 3, 4];
//        let tags_json_2 = CString::new(r#"{"tag1": "value_nem11", "tag2": "value_nem22", "~tag3": "value_nem33"}"#).unwrap();
//
//        let err = api::add_record(wallet.handle, type_.as_ptr(), id_2.as_ptr(), value_2.as_ptr(), value_2.len(), tags_json_2.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        // -- Search Records --
//        let query_json = json!({
//            "tag1": {"$in": ["value_nem1", "value_nem11"]},
//            "~tag3": {"$in": ["value_nem3", "value_nem33"]},
//            "$not": {
//                "tag2": "value_nem22"
//            }
//        });
//
//        let query_json = serde_json::to_string(&query_json).unwrap();
//        let query_json = CString::new(query_json).unwrap();
//
//        let options_json = CString::new("{}").unwrap();
//
//        let mut search_handle: i32 = -1;
//
//        let err = api::search_records(wallet.handle, type_.as_ptr(), query_json.as_ptr(), options_json.as_ptr(), &mut search_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut total_count = 0;
//        let err = api::get_search_total_count(wallet.handle, search_handle, &mut total_count);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let mut record_handle = -1;
//        let err = api::fetch_search_next_record(wallet.handle, search_handle, &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        // Check type
//        let mut type_p: *const c_char = ptr::null_mut();
//        let err = api::get_record_type(wallet.handle, record_handle, &mut type_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        // Check id/name
//        let mut id_p: *const c_char = ptr::null_mut();
//        let err = api::get_record_id(wallet.handle, record_handle, &mut id_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { CStr::from_ptr(id_p) }.to_owned(), id_1);
//
//        // Check value
//        let mut value_p: *const u8 = ptr::null_mut();
//        let mut value_len_p = 0;
//        let err = api::get_record_value(wallet.handle, record_handle, &mut value_p, &mut value_len_p);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(unsafe { slice::from_raw_parts(value_p, value_len_p) }, value_1.as_slice());
//
//        // Check tags
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//        let err = api::get_record_tags(wallet.handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let err = api::free_search(wallet.handle, search_handle);
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_search_records_fetch_none() {
//        let wallet = TestWallet::new_default(false);
//
//        let type_ = CString::new(random_string(10)).unwrap();
//
//        // -- Add records --
//        let id_1 = CString::new(random_name()).unwrap();
//        let value_1 = vec![1, 2, 3, 4];
//        let tags_json_1 = CString::new(r#"{"tag1": "value_nem1", "tag2": "value_nem2", "~tag3": "value_nem3"}"#).unwrap();
//
//        let err = api::add_record(wallet.handle, type_.as_ptr(), id_1.as_ptr(), value_1.as_ptr(), value_1.len(), tags_json_1.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        let id_2 = CString::new(random_name()).unwrap();
//        let value_2 = vec![1, 2, 3, 4];
//        let tags_json_2 = CString::new(r#"{"tag1": "value_nem11", "tag2": "value_nem22", "~tag3": "value_nem33"}"#).unwrap();
//
//        let err = api::add_record(wallet.handle, type_.as_ptr(), id_2.as_ptr(), value_2.as_ptr(), value_2.len(), tags_json_2.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//
//        // -- Search Records --
//        let query_json = json!({
//            "tag1": {"$in": ["value_nem1", "value_nem11"]},
//            "~tag3": {"$in": ["value_nem3", "value_nem33"]},
//            "$not": {
//                "tag2": "value_nem22"
//            }
//        });
//
//        let query_json = serde_json::to_string(&query_json).unwrap();
//        let query_json = CString::new(query_json).unwrap();
//
//        let options_json = search_options(false, false, false, false, false);
//
//        let mut search_handle: i32 = -1;
//
//        let err = api::search_records(wallet.handle, type_.as_ptr(), query_json.as_ptr(), options_json.as_ptr(), &mut search_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut total_count = 0;
//        let err = api::get_search_total_count(wallet.handle, search_handle, &mut total_count);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let mut record_handle = -1;
//        let err = api::fetch_search_next_record(wallet.handle, search_handle, &mut record_handle);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let err = api::free_search(wallet.handle, search_handle);
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_search_records_number_of_records_with_count() {
//        let wallet = TestWallet::new_default(false);
//
//        let num_of_records = 5;
//
//        let mut record_ids: Vec<CString> = Vec::new();
//
//        let type_ = CString::new(random_string(10)).unwrap();
//
//        // -- Add records --
//        for _i in 0..num_of_records {
//            let id = CString::new(random_name()).unwrap();
//            let value = vec![1, 2, 3, 4];
//            let tags_json = CString::new(r#"{"tag1": "value1", "tag2": "value2", "~tag3": "value3"}"#).unwrap();
//
//            let err = api::add_record(wallet.handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
//            assert_eq!(err, ErrorCode::Success);
//
//            record_ids.push(id);
//        }
//
//        // -- Search Records --
//        let query_json = json!({
//            "tag1": "value1",
//            "tag2": "value2",
//            "~tag3": "value3",
//            "$not": {
//                "tag1": "value11",
//                "tag2": "value22",
//                "~tag3": "value33",
//            }
//        });
//        let query_json = serde_json::to_string(&query_json).unwrap();
//        let query_json = CString::new(query_json).unwrap();
//
//        let options_json = search_options(true, true, true, true, true);
//
//        let mut search_handle: i32 = -1;
//        let err = api::search_records(wallet.handle, type_.as_ptr(), query_json.as_ptr(), options_json.as_ptr(), &mut search_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut total_count = 0;
//        let err = api::get_search_total_count(wallet.handle, search_handle, &mut total_count);
//        assert_eq!(err, ErrorCode::Success);
//        assert_eq!(total_count, num_of_records);
//
//        let mut record_handle = -1;
//        for _i in 0..num_of_records {
//            let err = api::fetch_search_next_record(wallet.handle, search_handle, &mut record_handle);
//            assert_eq!(err, ErrorCode::Success);
//        }
//
//        let err = api::fetch_search_next_record(wallet.handle, search_handle, &mut record_handle);
//        assert_eq!(err, ErrorCode::ItemNotFound);
//
//        let err = api::fetch_search_next_record(wallet.handle, search_handle, &mut record_handle);
//        assert_eq!(err, ErrorCode::ItemNotFound);
//
//        let err = api::free_search(wallet.handle, search_handle);
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_search_records_number_of_records_without_count() {
//        let wallet = TestWallet::new_default(false);
//
//        let num_of_records = 5;
//
//        let mut record_ids: Vec<CString> = Vec::new();
//
//        let type_ = CString::new(random_string(10)).unwrap();
//
//        // -- Add records --
//        for _i in 0..num_of_records {
//            let id = CString::new(random_name()).unwrap();
//            let value = vec![1, 2, 3, 4];
//            let tags_json = CString::new(r#"{"tag1": "value1", "tag2": "value2", "~tag3": "value3"}"#).unwrap();
//
//            let err = api::add_record(wallet.handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
//            assert_eq!(err, ErrorCode::Success);
//
//            record_ids.push(id);
//        }
//
//        // -- Search Records --
//        let query_json = json!({
//            "tag1": "value1",
//            "tag2": "value2",
//            "~tag3": "value3",
//            "$not": {
//                "tag1": "value11",
//                "tag2": "value22",
//                "~tag3": "value33",
//            }
//        });
//        let query_json = serde_json::to_string(&query_json).unwrap();
//        let query_json = CString::new(query_json).unwrap();
//
//        let options_json = search_options(true, false, true, true, true);
//
//        let mut search_handle: i32 = -1;
//        let err = api::search_records(wallet.handle, type_.as_ptr(), query_json.as_ptr(), options_json.as_ptr(), &mut search_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut total_count = 0;
//        let err = api::get_search_total_count(wallet.handle, search_handle, &mut total_count);
//        assert_eq!(err, ErrorCode::InvalidState);
//
//        let mut record_handle = -1;
//        for _i in 0..num_of_records {
//            let err = api::fetch_search_next_record(wallet.handle, search_handle, &mut record_handle);
//            assert_eq!(err, ErrorCode::Success);
//        }
//
//        let err = api::fetch_search_next_record(wallet.handle, search_handle, &mut record_handle);
//        assert_eq!(err, ErrorCode::ItemNotFound);
//
//        let err = api::fetch_search_next_record(wallet.handle, search_handle, &mut record_handle);
//        assert_eq!(err, ErrorCode::ItemNotFound);
//
//        let err = api::free_search(wallet.handle, search_handle);
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_search_records_invalid_query_format() {
//        let wallet = TestWallet::new_default(false);
//
//        let type_ = CString::new(random_string(10)).unwrap();
//        let query_json = CString::new("").unwrap();
//        let options_json = CString::new("{}").unwrap();
//        let mut search_handle: i32 = -1;
//
//        let err = api::search_records(wallet.handle, type_.as_ptr(), query_json.as_ptr(), options_json.as_ptr(), &mut search_handle);
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_search_records_invalid_options_format() {
//        let wallet = TestWallet::new_default(false);
//
//        let type_ = CString::new(random_string(10)).unwrap();
//        let query_json = CString::new("{}").unwrap();
//        let options_json = CString::new("").unwrap();
//        let mut search_handle: i32 = -1;
//
//        let err = api::search_records(wallet.handle, type_.as_ptr(), query_json.as_ptr(), options_json.as_ptr(), &mut search_handle);
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_search_records_invalid_query_eq_with_non_string_arg() {
//        let wallet = TestWallet::new_default(false);
//
//        let type_ = CString::new(random_string(10)).unwrap();
//
//        let query_json = json!({
//
//            "tag_name": 1
//
//        });
//
//        let query_json = serde_json::to_string(&query_json).unwrap();
//        let query_json = CString::new(query_json).unwrap();
//        let options_json = CString::new("{}").unwrap();
//        let mut search_handle: i32 = -1;
//
//        let err = api::search_records(wallet.handle, type_.as_ptr(), query_json.as_ptr(), options_json.as_ptr(), &mut search_handle);
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_search_records_invalid_query_neq_with_non_string_arg() {
//        let wallet = TestWallet::new_default(false);
//
//        let type_ = CString::new(random_string(10)).unwrap();
//
//        let query_json = json!({
//
//            "tag_name": {"$neq": 1}
//
//        });
//
//        let query_json = serde_json::to_string(&query_json).unwrap();
//        let query_json = CString::new(query_json).unwrap();
//        let options_json = CString::new("{}").unwrap();
//        let mut search_handle: i32 = -1;
//
//        let err = api::search_records(wallet.handle, type_.as_ptr(), query_json.as_ptr(), options_json.as_ptr(), &mut search_handle);
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_search_records_invalid_query_gt_with_non_string_arg() {
//        let wallet = TestWallet::new_default(false);
//
//        let type_ = CString::new(random_string(10)).unwrap();
//
//        let query_json = json!({
//
//            "k3": {"$gt": 1}
//
//        });
//
//        let query_json = serde_json::to_string(&query_json).unwrap();
//        let query_json = CString::new(query_json).unwrap();
//        let options_json = CString::new("{}").unwrap();
//        let mut search_handle: i32 = -1;
//
//        let err = api::search_records(wallet.handle, type_.as_ptr(), query_json.as_ptr(), options_json.as_ptr(), &mut search_handle);
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_search_records_invalid_query_gte_with_non_string_arg() {
//        let wallet = TestWallet::new_default(false);
//
//        let type_ = CString::new(random_string(10)).unwrap();
//
//        let query_json = json!({
//
//            "k3": {"$gte": 1}
//
//        });
//
//        let query_json = serde_json::to_string(&query_json).unwrap();
//        let query_json = CString::new(query_json).unwrap();
//        let options_json = CString::new("{}").unwrap();
//        let mut search_handle: i32 = -1;
//
//        let err = api::search_records(wallet.handle, type_.as_ptr(), query_json.as_ptr(), options_json.as_ptr(), &mut search_handle);
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_search_records_invalid_query_lt_with_non_string_arg() {
//        let wallet = TestWallet::new_default(false);
//
//        let type_ = CString::new(random_string(10)).unwrap();
//
//        let query_json = json!({
//
//            "k3": {"$lt": 1}
//
//        });
//
//        let query_json = serde_json::to_string(&query_json).unwrap();
//        let query_json = CString::new(query_json).unwrap();
//        let options_json = CString::new("{}").unwrap();
//        let mut search_handle: i32 = -1;
//
//        let err = api::search_records(wallet.handle, type_.as_ptr(), query_json.as_ptr(), options_json.as_ptr(), &mut search_handle);
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_search_records_invalid_query_lte_with_non_string_arg() {
//        let wallet = TestWallet::new_default(false);
//
//        let type_ = CString::new(random_string(10)).unwrap();
//
//        let query_json = json!({
//
//            "k3": {"$lte":  1}
//
//        });
//
//        let query_json = serde_json::to_string(&query_json).unwrap();
//        let query_json = CString::new(query_json).unwrap();
//        let options_json = CString::new("{}").unwrap();
//        let mut search_handle: i32 = -1;
//
//        let err = api::search_records(wallet.handle, type_.as_ptr(), query_json.as_ptr(), options_json.as_ptr(), &mut search_handle);
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_search_records_invalid_query_like_with_non_string_arg() {
//        let wallet = TestWallet::new_default(false);
//
//        let type_ = CString::new(random_string(10)).unwrap();
//
//        let query_json = json!({
//
//            "k2": {"$like": 1},
//
//        });
//
//        let query_json = serde_json::to_string(&query_json).unwrap();
//        let query_json = CString::new(query_json).unwrap();
//        let options_json = CString::new("{}").unwrap();
//        let mut search_handle: i32 = -1;
//
//        let err = api::search_records(wallet.handle, type_.as_ptr(), query_json.as_ptr(), options_json.as_ptr(), &mut search_handle);
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_search_records_invalid_query_not_with_non_string_arg() {
//        let wallet = TestWallet::new_default(false);
//
//        let type_ = CString::new(random_string(10)).unwrap();
//
//        let query_json = json!({
//
//            "$in": [1, "value_nem33"]
//
//        });
//
//        let query_json = serde_json::to_string(&query_json).unwrap();
//        let query_json = CString::new(query_json).unwrap();
//        let options_json = CString::new("{}").unwrap();
//        let mut search_handle: i32 = -1;
//
//        let err = api::search_records(wallet.handle, type_.as_ptr(), query_json.as_ptr(), options_json.as_ptr(), &mut search_handle);
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_search_records_invalid_query_or_with_non_string_arg() {
//        let wallet = TestWallet::new_default(false);
//
//        let type_ = CString::new(random_string(10)).unwrap();
//
//        let query_json = json!({
//
//            "$or": [
//                {
//                    "k2": 1,
//                },
//            ],
//
//        });
//
//        let query_json = serde_json::to_string(&query_json).unwrap();
//        let query_json = CString::new(query_json).unwrap();
//        let options_json = CString::new("{}").unwrap();
//        let mut search_handle: i32 = -1;
//
//        let err = api::search_records(wallet.handle, type_.as_ptr(), query_json.as_ptr(), options_json.as_ptr(), &mut search_handle);
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_search_records_invalid_query_in_with_non_string_arg() {
//        let wallet = TestWallet::new_default(false);
//
//        let type_ = CString::new(random_string(10)).unwrap();
//
//        let query_json = json!({
//
//            "$in": [1, "value_nem33"]
//
//        });
//
//        let query_json = serde_json::to_string(&query_json).unwrap();
//        let query_json = CString::new(query_json).unwrap();
//        let options_json = CString::new("{}").unwrap();
//        let mut search_handle: i32 = -1;
//
//        let err = api::search_records(wallet.handle, type_.as_ptr(), query_json.as_ptr(), options_json.as_ptr(), &mut search_handle);
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_search_records_invalid_query_gt_with_encrypted_tag_name() {
//        let wallet = TestWallet::new_default(false);
//
//        let type_ = CString::new(random_string(10)).unwrap();
//
//        let query_json = json!({
//
//            "k3": {"$gt": "a"}
//
//        });
//
//        let query_json = serde_json::to_string(&query_json).unwrap();
//        let query_json = CString::new(query_json).unwrap();
//        let options_json = CString::new("{}").unwrap();
//        let mut search_handle: i32 = -1;
//
//        let err = api::search_records(wallet.handle, type_.as_ptr(), query_json.as_ptr(), options_json.as_ptr(), &mut search_handle);
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_search_records_invalid_query_gte_with_encrypted_tag_name() {
//        let wallet = TestWallet::new_default(false);
//
//        let type_ = CString::new(random_string(10)).unwrap();
//
//        let query_json = json!({
//
//            "k3": {"$gte": "a"}
//
//        });
//
//        let query_json = serde_json::to_string(&query_json).unwrap();
//        let query_json = CString::new(query_json).unwrap();
//        let options_json = CString::new("{}").unwrap();
//        let mut search_handle: i32 = -1;
//
//        let err = api::search_records(wallet.handle, type_.as_ptr(), query_json.as_ptr(), options_json.as_ptr(), &mut search_handle);
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_search_records_invalid_query_lt_with_encrypted_tag_name() {
//        let wallet = TestWallet::new_default(false);
//
//        let type_ = CString::new(random_string(10)).unwrap();
//
//        let query_json = json!({
//
//            "k3": {"$lt": "a"}
//
//        });
//
//        let query_json = serde_json::to_string(&query_json).unwrap();
//        let query_json = CString::new(query_json).unwrap();
//        let options_json = CString::new("{}").unwrap();
//        let mut search_handle: i32 = -1;
//
//        let err = api::search_records(wallet.handle, type_.as_ptr(), query_json.as_ptr(), options_json.as_ptr(), &mut search_handle);
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_search_records_invalid_query_lte_with_encrypted_tag_name() {
//        let wallet = TestWallet::new_default(false);
//
//        let type_ = CString::new(random_string(10)).unwrap();
//
//        let query_json = json!({
//
//            "k3": {"$lte": "a"}
//
//        });
//
//        let query_json = serde_json::to_string(&query_json).unwrap();
//        let query_json = CString::new(query_json).unwrap();
//        let options_json = CString::new("{}").unwrap();
//        let mut search_handle: i32 = -1;
//
//        let err = api::search_records(wallet.handle, type_.as_ptr(), query_json.as_ptr(), options_json.as_ptr(), &mut search_handle);
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_search_records_invalid_query_like_with_encrypted_tag_name() {
//        let wallet = TestWallet::new_default(false);
//
//        let type_ = CString::new(random_string(10)).unwrap();
//
//        let query_json = json!({
//
//            "k2": {"$like": "like_target"},
//
//        });
//
//        let query_json = serde_json::to_string(&query_json).unwrap();
//        let query_json = CString::new(query_json).unwrap();
//        let options_json = CString::new("{}").unwrap();
//        let mut search_handle: i32 = -1;
//
//        let err = api::search_records(wallet.handle, type_.as_ptr(), query_json.as_ptr(), options_json.as_ptr(), &mut search_handle);
//        assert_eq!(err, ErrorCode::InvalidStructure);
//    }
//
//    #[test]
//    fn test_search_records_invalid_storage_handle() {
//        let wallet = TestWallet::new_default(true);
//
//        let type_ = CString::new(random_string(10)).unwrap();
//        let query_json = CString::new("{}").unwrap();
//        let options_json = CString::new("{}").unwrap();
//        let mut search_handle: i32 = -1;
//
//        let err = api::search_records(wallet.handle, type_.as_ptr(), query_json.as_ptr(), options_json.as_ptr(), &mut search_handle);
//        assert_eq!(err, ErrorCode::InvalidState);
//    }
//
//    /** Search All Records Tests */
//
//    #[test]
//    fn test_search_all_records() {
//        let wallet = TestWallet::new_default(false);
//
//        let num_of_records: i32 = 10;
//        let mut record_ids: Vec<CString> = Vec::new();
//
//        // -- Add records --
//        for _i in 0..num_of_records {
//            let record = TestRecord::new_default(true);
//            wallet.add_record(&record);
//            record_ids.push(record.id);
//        }
//
//        // -- Search Records --
//        let mut search_handle: i32 = -1;
//
//        let err = api::search_all_records(wallet.handle, &mut search_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut record_handle = -1;
//
//        for _i in 0..num_of_records {
//            let err = api::fetch_search_next_record(wallet.handle, search_handle, &mut record_handle);
//            assert_eq!(err, ErrorCode::Success);
//        }
//
//        let err = api::fetch_search_next_record(wallet.handle, search_handle, &mut record_handle);
//        assert_eq!(err, ErrorCode::ItemNotFound);
//
//        let err = api::fetch_search_next_record(wallet.handle, search_handle, &mut record_handle);
//        assert_eq!(err, ErrorCode::ItemNotFound);
//
//        let err = api::free_search(wallet.handle, search_handle);
//        assert_eq!(err, ErrorCode::Success);
//    }
//
//    #[test]
//    fn test_search_all_records_invalid_storage_handle() {
//        let wallet = TestWallet::new_default(true);
//
//        let type_ = CString::new(random_string(10)).unwrap();
//        let query_json = CString::new("{}").unwrap();
//        let options_json = CString::new("{}").unwrap();
//        let mut search_handle: i32 = -1;
//
//        let err = api::search_records(wallet.handle, type_.as_ptr(), query_json.as_ptr(), options_json.as_ptr(), &mut search_handle);
//        assert_eq!(err, ErrorCode::InvalidState);
//    }
//
//    #[test]
//    fn fetch_search_next_record_invalid_search_handle() {
//        let wallet = TestWallet::new_default(false);
//
//        let search_handle: i32 = -1;
//        let mut record_handle: i32 = -1;
//
//        let err = api::fetch_search_next_record(wallet.handle, search_handle, &mut record_handle);
//        assert_eq!(err, ErrorCode::InvalidState);
//    }
//
//    #[test]
//    fn test_search_free_invalid_storage_handle() {
//        let wallet = TestWallet::new_default(true);
//
//        let search_handle: i32 = -1;
//
//        let err = api::free_search(wallet.handle, search_handle);
//        assert_eq!(err, ErrorCode::InvalidState);
//    }
//
//    #[test]
//    fn test_search_free_invalid_search_handle() {
//        let wallet = TestWallet::new_default(false);
//
//        let search_handle: i32 = -1;
//
//        let err = api::free_search(wallet.handle, search_handle);
//        assert_eq!(err, ErrorCode::InvalidState);
//    }
}

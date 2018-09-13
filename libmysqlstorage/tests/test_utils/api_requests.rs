extern crate mysqlstorage;
extern crate serde_json;

use mysqlstorage::api as api;
use mysqlstorage::errors::error_code::ErrorCode;

use test_utils::config::Config;

use std::ffi::{CString};

pub mod api_requests {

    use super::*;
    use std::ptr;
    use std::os::raw::c_char;

    const RECORD_TYPE: &'static str = "test-type";

    lazy_static! {
        static ref TEST_CONFIG: Config = Config::new();
    }


    pub fn open_storage(wallet_name: &String) -> i32 {
        let name = CString::new(wallet_name.clone()).unwrap();
        let config = CString::new(TEST_CONFIG.get_config()).unwrap();
        let credentials = CString::new(TEST_CONFIG.get_credentials()).unwrap();
        let mut handle: i32 = -1;

        let err = api::open_storage(name.as_ptr(), config.as_ptr(), credentials.as_ptr(), &mut handle);

        assert_eq!(err, ErrorCode::Success);
        handle
    }

    pub fn open_and_close_storage(wallet_name: &String){
        let handle = open_storage(&wallet_name);
        let err = api::close_storage(handle);
        assert_eq!(err, ErrorCode::Success);
    }

    pub fn create_wallet(wallet_name: &String) {
        let name = CString::new(wallet_name.clone()).unwrap();

        let config = CString::new(TEST_CONFIG.get_config()).unwrap();
        let credentials = CString::new(TEST_CONFIG.get_credentials()).unwrap();
        let metadata = CString::new("metadata").unwrap();

        let err = api::create_storage(name.as_ptr(), config.as_ptr(), credentials.as_ptr(), metadata.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    pub fn set_metadata(handle: i32, new_metadata: &String){
        let new_metadata_cstring = CString::new(new_metadata.clone()).unwrap();

        let err = api::set_metadata(handle, new_metadata_cstring.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    pub fn get_metadata(handle: i32){
        let mut metadata_handle = -1;
        let mut metadata_ptr: *const c_char = ptr::null_mut();

        let err = api::get_metadata(handle, &mut metadata_ptr, &mut metadata_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = api::free_metadata(handle, metadata_handle);
        assert_eq!(err, ErrorCode::Success);
    }

    pub fn delete_wallet(wallet_name: &String){
        let name = CString::new(wallet_name.clone()).unwrap();
        let config = CString::new(TEST_CONFIG.get_config()).unwrap();
        let credentials = CString::new(TEST_CONFIG.get_credentials()).unwrap();

        let err = api::delete_storage(name.as_ptr(), config.as_ptr(), credentials.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    pub fn add_record(handle: i32, record_id: &String, record_value: &Vec<u8>, tags: &String) {
        let record_id = CString::new(record_id.clone()).unwrap();
        let type_ = CString::new(RECORD_TYPE).unwrap();
        let tags: String = format!(r##"{}"##, tags.clone());
        let tags_json = CString::new(tags).unwrap();

        let err = api::add_record(handle, type_.as_ptr(), record_id.as_ptr(), record_value.as_ptr(), record_value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    pub fn get_record_with_details(handle: i32, record_id: &String){
        let type_ = CString::new(RECORD_TYPE).unwrap();
        let record_id = CString::new(record_id.clone()).unwrap();
        let options_json = CString::new(r##"{"fetch_value": true, "fetch_tags": true}"##).unwrap();
        let mut record_handle = -1;
        let mut id_p: *const c_char = ptr::null_mut();
        let mut value_p: *const u8 = ptr::null_mut();
        let mut value_len_p = 0;
        let mut tags_json_p: *const c_char = ptr::null_mut();

        let err = api::get_record(handle, type_.as_ptr(), record_id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = api::get_record_id(handle, record_handle, &mut id_p);
        assert_eq!(err, ErrorCode::Success);

        let err = api::get_record_value(handle, record_handle, &mut value_p, &mut value_len_p);
        assert_eq!(err, ErrorCode::Success);

        let err = api::get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::Success);

        let err = api::free_record(handle, record_handle);
        assert_eq!(err, ErrorCode::Success);
    }

    pub fn delete_record(handle: i32, record_id: &String){
        let type_ = CString::new(RECORD_TYPE).unwrap();
        let record_id = CString::new(record_id.clone()).unwrap();

        let err = api::delete_record(handle, type_.as_ptr(), record_id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    pub fn update_record_value(handle: i32, record_id: &String, new_record_value: &Vec<u8>) {
        let record_id = CString::new(record_id.clone()).unwrap();
        let type_ = CString::new(RECORD_TYPE).unwrap();

        let err = api::update_record_value(handle, type_.as_ptr(), record_id.as_ptr(), new_record_value.as_ptr(), new_record_value.len());
        assert_eq!(err, ErrorCode::Success);
    }

    pub fn add_record_tags(handle: i32, record_id: &String, tags: &str) {
        let record_id = CString::new(record_id.clone()).unwrap();
        let type_ = CString::new(RECORD_TYPE).unwrap();
        let tags_json = CString::new(tags).unwrap();

        let err = api::add_record_tags(handle, type_.as_ptr(), record_id.as_ptr(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    pub fn update_record_tags(handle: i32, record_id: &String, tags: &str) {
        let record_id = CString::new(record_id.clone()).unwrap();
        let type_ = CString::new(RECORD_TYPE).unwrap();
        let tags_json = CString::new(tags).unwrap();

        let err = api::update_record_tags(handle, type_.as_ptr(), record_id.as_ptr(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    pub fn delete_record_tags(handle: i32, record_id: &String, tags: &str) {
        let record_id = CString::new(record_id.clone()).unwrap();
        let type_ = CString::new(RECORD_TYPE).unwrap();
        let tags_json = CString::new(tags).unwrap();

        let err = api::delete_record_tags(handle, type_.as_ptr(), record_id.as_ptr(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    pub fn search_records(handle: i32, query_json: &str){
        let type_ = CString::new(RECORD_TYPE).unwrap();
        let query_json = CString::new(query_json).unwrap();
        let options_json = CString::new(r##"{"retrieveRecords": true, "retrieveTotalCount": true, "retrieveType": true, "retrieveValue": true, "retrieveTags": true}"##).unwrap();
        let mut search_handle: i32 = -1;

        let err = api::search_records(handle, type_.as_ptr(), query_json.as_ptr(), options_json.as_ptr(), &mut search_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = api::free_search(handle, search_handle);
        assert_eq!(err, ErrorCode::Success);
    }

    pub fn search_all_records(handle: i32){
        let mut search_handle: i32 = -1;

        let err = api::search_all_records(handle, &mut search_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = api::free_search(handle, search_handle);
        assert_eq!(err, ErrorCode::Success);
    }
}
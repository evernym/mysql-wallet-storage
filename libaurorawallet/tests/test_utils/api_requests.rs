extern crate aurorastorage;

use aurorastorage::api as api;
use aurorastorage::errors::error_code::ErrorCode;

use test_utils::config::{ConfigType, Config};
use test_utils::helper_functions::random_string;

use std::ffi::{CString};

pub mod api_requests {

    use super::*;
    use std::ffi::CStr;
    use std::slice;
    use std::ptr;
    use std::os::raw::c_char;

    const ITEM_TYPE: &'static str = "test-type";

    lazy_static! {
        static ref TEST_CONFIG: Config = Config::new(ConfigType::QA);
    }


    pub fn open_storage(wallet_name: &String) -> i32 {
        let name = CString::new(wallet_name.clone()).unwrap();
        let config = CString::new(TEST_CONFIG.get_config()).unwrap();
        let runtime_config = CString::new(TEST_CONFIG.get_runtime_config()).unwrap();
        let credentials = CString::new(TEST_CONFIG.get_credentials()).unwrap();
        let mut handle: i32 = -1;

        let err = api::open_storage(name.as_ptr(), config.as_ptr(), runtime_config.as_ptr(), credentials.as_ptr(), &mut handle);

        assert_eq!(err, ErrorCode::Success);
        handle
    }

    pub fn create_wallet(wallet_name: &String) {
        let name = CString::new(wallet_name.clone()).unwrap();

        let config = CString::new(TEST_CONFIG.get_config()).unwrap();
        let credentials = CString::new(TEST_CONFIG.get_credentials()).unwrap();
        let metadata = CString::new("metadata").unwrap();

        let err = api::create_storage(name.as_ptr(), config.as_ptr(), credentials.as_ptr(), metadata.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    pub fn delete_wallet(wallet_name: &String){
        let name = CString::new(wallet_name.clone()).unwrap();
        let config = CString::new(TEST_CONFIG.get_config()).unwrap();
        let credentials = CString::new(TEST_CONFIG.get_credentials()).unwrap();
        let err = api::delete_storage(name.as_ptr(), config.as_ptr(), credentials.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }
    pub fn add_record(wallet_name: &String, item_id: &String, item_value: &Vec<u8>, tags: &String) {
        let handle = open_storage(wallet_name);

        let item_id = CString::new(item_id.clone()).unwrap();
        let type_ = CString::new(ITEM_TYPE).unwrap();
        let tags: String = format!(r##"{}"##, tags.clone());
        let tags_json = CString::new(tags).unwrap();
        let err = api::add_record(handle, type_.as_ptr(), item_id.as_ptr(), item_value.as_ptr(), item_value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

//    pub fn add_record_with_tags(wallet_name: &String, item_id: &String, item_value: &Vec<u8>, tags: &String) {
//        let handle = open_storage(wallet_name);
//
//        let item_id = CString::new(item_id.clone()).unwrap();
//        let type_ = CString::new(ITEM_TYPE).unwrap();
//        let tags: String = format!(r##"{}"##, tags.clone());
//        let tags_json = CString::new(tags).unwrap();
//
//        let err = api::add_record(handle, type_.as_ptr(), item_id.as_ptr(), item_value.as_ptr(), item_value.len(), tags_json.as_ptr());
//        assert_eq!(err, ErrorCode::Success);
//    }

    pub fn get_record_with_details(wallet_name: &String, item_id: &String){
        let handle = open_storage(wallet_name);
        let type_ = CString::new(ITEM_TYPE).unwrap();
        let item_id = CString::new(item_id.clone()).unwrap();
        let options_json = CString::new(r##"{"fetch_value": true, "fetch_tags": true}"##).unwrap();
        let mut record_handle = -1;
        let mut id_p: *const c_char = ptr::null_mut();
        let mut value_p: *const u8 = ptr::null_mut();
        let mut value_len_p = 0;
        let mut tags_json_p: *const c_char = ptr::null_mut();

        let err = api::get_record(handle, type_.as_ptr(), item_id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = api::get_record_id(handle, record_handle, &mut id_p);
        assert_eq!(err, ErrorCode::Success);

        let err = api::get_record_value(handle, record_handle, &mut value_p, &mut value_len_p);
        assert_eq!(err, ErrorCode::Success);

        let err = api::get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::Success);
    }

    pub fn delete_record(wallet_name: &String, item_id: &String){
        let handle = open_storage(wallet_name);
        let type_ = CString::new(ITEM_TYPE).unwrap();
        let item_id = CString::new(item_id.clone()).unwrap();

        let err = api::delete_record(handle, type_.as_ptr(), item_id.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    pub fn update_record_value(wallet_name: &String, item_id: &String, new_item_value: &Vec<u8>) {
        let handle = open_storage(wallet_name);

        let item_id = CString::new(item_id.clone()).unwrap();
        let type_ = CString::new(ITEM_TYPE).unwrap();
        let err = api::update_record_value(handle, type_.as_ptr(), item_id.as_ptr(), new_item_value.as_ptr(), new_item_value.len());
        assert_eq!(err, ErrorCode::Success);
    }

    pub fn add_record_tags(wallet_name: &String, item_id: &String, tags: &String) {
        let handle = open_storage(wallet_name);
        let item_id = CString::new(item_id.clone()).unwrap();
        let type_ = CString::new(ITEM_TYPE).unwrap();
        let tags: String = format!(r##"{}"##, tags.clone());
        let tags_json = CString::new(tags).unwrap();
        let err = api::add_record_tags(handle, type_.as_ptr(), item_id.as_ptr(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

    }

    pub fn update_record_tags(wallet_name: &String, item_id: &String, tags: &String) {
        let handle = open_storage(wallet_name);
        let item_id = CString::new(item_id.clone()).unwrap();
        let type_ = CString::new(ITEM_TYPE).unwrap();
        let tags: String = format!(r##"{}"##, tags.clone());
        let tags_json = CString::new(tags).unwrap();
        let err = api::update_record_tags(handle, type_.as_ptr(), item_id.as_ptr(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

    }

    pub fn delete_record_tags(wallet_name: &String, item_id: &String, tags: &Vec<String>) {
        let handle = open_storage(wallet_name);
        let item_id = CString::new(item_id.clone()).unwrap();
        let type_ = CString::new(ITEM_TYPE).unwrap();
        let tags: String = tags.join(",");
        let tags_json = CString::new(tags).unwrap();
        let err = api::delete_record_tags(handle, type_.as_ptr(), item_id.as_ptr(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

    }
}
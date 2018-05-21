// Local dependencies
extern crate libaurorawallet;

pub mod test_utils;
use test_utils::config::{ConfigType, Config};

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate lazy_static;

use libaurorawallet::api as api;
use libaurorawallet::errors::error_code::ErrorCode;
use std::ffi::CString;

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


    #[test]
    pub fn indy_register_wallet_storage_test() {

    }
}
// Local dependencies
extern crate mysqlstorage;

use mysqlstorage::api as api;
use mysqlstorage::errors::error_code::ErrorCode;

mod test_utils;
use test_utils::config::{ConfigType, Config};

// External dependencies
extern crate libc;
use libc::c_char;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate lazy_static;

use std::ptr;
use std::ffi::CString;

use std::cmp::max;
use std::thread;
use std::time::{Duration, Instant};

///
/// Module for testing general performance of MySQL Plugin functions.
///
/// # Main components
///  * `driver functions` - functions that prepare input and call Storage Plugin API methods
///  * `bench()` - function for benchmarking
///  * `perf_runner()` - defines which `driver function` will be benchmarked via `bench()`
///  * `THREAD_CNT` - defines how many threads will be used for the test
///  * `OPERATIONS_CNT` - defines how many times the `driver function` will be called by a single thread
///
/// # General Idea
///  This module defines driver functions that call appropriate Storage Plugin functions.
///  Performance is tested by passing these functions to `bench()` function that runs them,
///  and gather performance metrics. `perf_runner()` defines which driver methods will be tested,
///  and passes them to `bench()`.
///
///  While being tested every function prepares the data for the function that follows. All driver
///  functions accept two parameters, thread_id and operation_id that allow each thread and each
///  operation to work without conflict. Thus the `chained` at the beginning of the module, as all
///  driver are basically `chained` in their execution.
///
///  All tests in this module are performed on wallets that contain a single item.
///
///  This module is useful for getting a general feel of performance/regression of API methods,
///  but it is not testing effects of real world load.
///
/// # Example Flow - Single thread, two operations
///  * create_wallet is called -> wallet_0_0 and wallet_0_1 are created
///  * add_record is called -> record_0_0 is created in wallet_0_0, and record_0_1 in wallet_0_1
///  * add_tags is called -> identical tags are added to record_0_0 and record_0_1
///  * etc...
///
/// # Example Output
///
///         Benchmarking Create Wallet
///  ================= Summary =================
///  Max Execution Time:      	Duration { secs: 0, nanos: 809019977 }
///  Operations Executed:     	10000
///  Sum of Exection times:   	Duration { secs: 58, nanos: 294541212 }
///  Total Duration:          	Duration { secs: 58, nanos: 333146322 }
///  Aprox TPS:               	172
///
/// # How to Execute
///
///  * If running from the console: run `cargo test --release --package mysqlstorage --test chained_perf_test chaned_perf_test::perf_runner -- --exact --nocapture`
///  * If using Pycharm: Run the `perf_runner()` test method - Right Click -> Run perf_runner
///
mod chaned_perf_test {

    use super::*;

    lazy_static! {
        static ref TEST_CONFIG: Config = Config::new(ConfigType::QA);
    }

    const THREAD_CNT: u64 = 1;
    const OPERATIONS_CNT: u64 = 2000;
    const ITEM_TYPE: &'static str = "demo-type";

    ///
    /// Helper function for establishing a connection to a wallet.
    ///
    /// Prepares all of the input parameters and calls the corresponding storage plugin api method.
    ///
    /// # Params
    ///  * wallet_name: String - name of the wallet that ought to be opened.
    ///
    fn open_storage(wallet_name: String) -> i32 {
        let name = CString::new(wallet_name).unwrap();
        let config = CString::new(TEST_CONFIG.get_config()).unwrap();
        let credentials = CString::new(TEST_CONFIG.get_credentials()).unwrap();
        let mut handle: i32 = -1;

        let err = api::open_storage(name.as_ptr(), config.as_ptr(), credentials.as_ptr(), &mut handle);

        assert_eq!(err, ErrorCode::Success);
        handle
    }

    ///
    /// `Create Wallet` driver function.
    ///
    /// Prepares all of the input parameters and calls the corresponding storage plugin api method.
    ///
    /// # Params
    ///  * x: u64 - ID of the calling thread (goes from 0 to THREAD_CNT)
    ///  * y: u64 - ID of the operation being performed by a thread (goes from 0 to OPERATION_COUNT)
    ///
    fn create_wallet(x: u64, y: u64) {
        let name = format!("wallet_{}_{}", x, y);
        let name = CString::new(name).unwrap();

        let config = CString::new(TEST_CONFIG.get_config()).unwrap();
        let credentials = CString::new(TEST_CONFIG.get_credentials()).unwrap();
        let metadata = CString::new("metadata").unwrap();

        let err = api::create_storage(name.as_ptr(), config.as_ptr(), credentials.as_ptr(), metadata.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    ///
    /// `Set Metadata` driver function.
    ///
    /// Prepares all of the input parameters and calls the corresponding storage plugin api method.
    ///
    /// # Params
    ///  * x: u64 - ID of the calling thread (goes from 0 to THREAD_CNT)
    ///  * y: u64 - ID of the operation being performed by a thread (goes from 0 to OPERATION_COUNT)
    ///
    fn set_metadata(x: u64, y: u64) {
        let wallet_name = format!("wallet_{}_{}", x, y);
        let handle = open_storage(wallet_name);

        let new_metadata = String::from("METADATA METADATA METADATA METADATA METADATA METADATA");
        let new_metadata_cstring = CString::new(new_metadata.clone()).unwrap();

        let err = api::set_metadata(handle, new_metadata_cstring.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = api::close_storage(handle);
        assert_eq!(err, ErrorCode::Success);
    }

    ///
    /// `Get Metadata` driver function.
    ///
    /// Prepares all of the input parameters and calls the corresponding storage plugin api method.
    ///
    /// # Params
    ///  * x: u64 - ID of the calling thread (goes from 0 to THREAD_CNT)
    ///  * y: u64 - ID of the operation being performed by a thread (goes from 0 to OPERATION_COUNT)
    ///
    fn get_metadata(x: u64, y: u64) {
        let wallet_name = format!("wallet_{}_{}", x, y);
        let handle = open_storage(wallet_name);

        let mut metadata_handle = -1;
        let mut metadata_ptr: *const c_char = ptr::null_mut();

        let err = api::get_metadata(handle, &mut metadata_ptr, &mut metadata_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = api::free_metadata(handle, metadata_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = api::close_storage(handle);
        assert_eq!(err, ErrorCode::Success);
    }

    ///
    /// Delete wallet driver function.
    ///
    /// Prepares all of the input parameters and calls the corresponding storage plugin api method.
    ///
    /// # Params
    ///  * x: u64 - ID of the calling thread (goes from 0 to THREAD_CNT)
    ///  * y: u64 - ID of the operation being performed by a thread (goes from 0 to OPERATION_COUNT)
    ///
    fn add_record(x: u64, y: u64) {
        let wallet_name = format!("wallet_{}_{}", x, y);
        let handle = open_storage(wallet_name);
        let id = format!("record_{}_{}", x, y);
        let id = CString::new(id).unwrap();
        let type_ =CString::new(ITEM_TYPE).unwrap();
        let value = vec![1, 2, 3, 4];
        let tags_json = CString::new(r##"{"tag1": "value1", "tag2": "value2", "tag3": "value3", "~tag4": "value4",  "~tag5": "value5", "~tag6": "value6"}"##).unwrap();
//        let tags_json = CString::new(r##"{}"##).unwrap();

        let err = api::add_record(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = api::close_storage(handle);
        assert_eq!(err, ErrorCode::Success);
    }

    ///
    /// `Get Record` driver function.
    ///
    /// Prepares all of the input parameters and calls the corresponding storage plugin api method.
    ///
    /// # Params
    ///  * x: u64 - ID of the calling thread (goes from 0 to THREAD_CNT)
    ///  * y: u64 - ID of the operation being performed by a thread (goes from 0 to OPERATION_COUNT)
    ///
    fn get_record(x: u64, y: u64) {
        let wallet_name = format!("wallet_{}_{}", x, y);
        let handle = open_storage(wallet_name);
        let type_ = CString::new(ITEM_TYPE).unwrap();
        let id = format!("record_{}_{}", x, y);
        let id = CString::new(id).unwrap();
        let options_json = CString::new(r##"{"retrieveValue": true, "retrieveTags": true}"##).unwrap();

        let mut record_handle = -1;
        let mut id_p: *const c_char = ptr::null_mut();
        let mut value_p: *const u8 = ptr::null_mut();
        let mut value_len_p = 0;
        let mut tags_json_p: *const c_char = ptr::null_mut();

        let err = api::get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = api::get_record_id(handle, record_handle, &mut id_p);
        assert_eq!(err, ErrorCode::Success);

        let err = api::get_record_value(handle, record_handle, &mut value_p, &mut value_len_p);
        assert_eq!(err, ErrorCode::Success);

        let err = api::get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::Success);

        let err = api::free_record(handle, record_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = api::close_storage(handle);
        assert_eq!(err, ErrorCode::Success);
    }

    ///
    /// `Update Value` driver function.
    ///
    /// Prepares all of the input parameters and calls the corresponding storage plugin api method.
    ///
    /// # Params
    ///  * x: u64 - ID of the calling thread (goes from 0 to THREAD_CNT)
    ///  * y: u64 - ID of the operation being performed by a thread (goes from 0 to OPERATION_COUNT)
    ///
    fn update_record_value(x: u64, y: u64) {
        let wallet_name = format!("wallet_{}_{}", x, y);
        let handle = open_storage(wallet_name);
        let type_ = CString::new(ITEM_TYPE).unwrap();
        let id = format!("record_{}_{}", x, y);
        let id = CString::new(id).unwrap();
        let new_value = vec![2, 5, 8, 13];

        let err = api::update_record_value(handle, type_.as_ptr(), id.as_ptr(), new_value.as_ptr(), new_value.len());
        assert_eq!(err, ErrorCode::Success);

        let err = api::close_storage(handle);
        assert_eq!(err, ErrorCode::Success);
    }

    ///
    /// `Add Tags` driver function.
    ///
    /// Prepares all of the input parameters and calls the corresponding storage plugin api method.
    ///
    /// # Params
    ///  * x: u64 - ID of the calling thread (goes from 0 to THREAD_CNT)
    ///  * y: u64 - ID of the operation being performed by a thread (goes from 0 to OPERATION_COUNT)
    ///
    fn add_record_tags(x: u64, y: u64) {
        let wallet_name = format!("wallet_{}_{}", x, y);
        let handle = open_storage(wallet_name);
        let type_ = CString::new(ITEM_TYPE).unwrap();
        let id = format!("record_{}_{}", x, y);
        let id = CString::new(id).unwrap();
        let tags_json = CString::new(r##"{"tag1": "new_value1", "new_encrypted": "ASDOPASFO==", "~new_plaintext": "as plain as it can be", "~age": 30}"##).unwrap();

        let err = api::add_record_tags(handle, type_.as_ptr(), id.as_ptr(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = api::close_storage(handle);
        assert_eq!(err, ErrorCode::Success);
    }

    ///
    /// `Update Tags` driver function.
    ///
    /// Prepares all of the input parameters and calls the corresponding storage plugin api method.
    ///
    /// # Params
    ///  * x: u64 - ID of the calling thread (goes from 0 to THREAD_CNT)
    ///  * y: u64 - ID of the operation being performed by a thread (goes from 0 to OPERATION_COUNT)
    ///
    fn update_record_tags(x: u64, y: u64) {
        let wallet_name = format!("wallet_{}_{}", x, y);
        let handle = open_storage(wallet_name);
        let type_ = CString::new(ITEM_TYPE).unwrap();
        let id = format!("record_{}_{}", x, y);
        let id = CString::new(id).unwrap();
        let tags_json = CString::new(r##"{"new_encrypted": "After update", "~new_plaintext": "After Update"}"##).unwrap();

        let err = api::update_record_tags(handle, type_.as_ptr(), id.as_ptr(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = api::close_storage(handle);
        assert_eq!(err, ErrorCode::Success);
    }

    ///
    /// `Search` wallet driver function.
    ///
    /// Prepares all of the input parameters and calls the corresponding storage plugin api method.
    ///
    /// # Params
    ///  * x: u64 - ID of the calling thread (goes from 0 to THREAD_CNT)
    ///  * y: u64 - ID of the operation being performed by a thread (goes from 0 to OPERATION_COUNT)
    ///
    fn search_records(x: u64, y: u64) {
        let wallet_name = format!("wallet_{}_{}", x, y);
        let handle = open_storage(wallet_name);

        let type_ = CString::new(ITEM_TYPE).unwrap();

        let query_json = json!({
            "new_encrypted": {
                "$in": ["After update", "After Update"]
                },
            "~new_plaintext": {
                "$in": ["After Update", "After update"]
                }
        });

        let query_json = serde_json::to_string(&query_json).unwrap();

        let query_json = CString::new(query_json).unwrap();

        let options_json = CString::new(r##"{"retrieveValue": true, "retrieveTags": true}"##).unwrap();

        let mut search_handle: i32 = -1;

        let err = api::search_records(handle, type_.as_ptr(), query_json.as_ptr(), options_json.as_ptr(), &mut search_handle);
        assert_eq!(err, ErrorCode::Success);

        let mut record_handle = -1;
        let err = api::fetch_search_next_record(handle, search_handle, &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = api::free_search(handle, search_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = api::close_storage(handle);
        assert_eq!(err, ErrorCode::Success);
    }

    ///
    /// `Search All` driver function.
    ///
    /// Prepares all of the input parameters and calls the corresponding storage plugin api method.
    ///
    /// # Params
    ///  * x: u64 - ID of the calling thread (goes from 0 to THREAD_CNT)
    ///  * y: u64 - ID of the operation being performed by a thread (goes from 0 to OPERATION_COUNT)
    ///
    fn search_all_records(x: u64, y: u64) {
        let wallet_name = format!("wallet_{}_{}", x, y);
        let handle = open_storage(wallet_name);

        let mut search_handle: i32 = -1;

        let err = api::search_all_records(handle, &mut search_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = api::free_search(handle, search_handle);
        assert_eq!(err, ErrorCode::Success);

        let err = api::close_storage(handle);
        assert_eq!(err, ErrorCode::Success);
    }

    ///
    /// `Delete Record Tags` driver function.
    ///
    /// Prepares all of the input parameters and calls the corresponding storage plugin api method.
    ///
    /// # Params
    ///  * x: u64 - ID of the calling thread (goes from 0 to THREAD_CNT)
    ///  * y: u64 - ID of the operation being performed by a thread (goes from 0 to OPERATION_COUNT)
    ///
    fn delete_record_tags(x: u64, y: u64) {
        let wallet_name = format!("wallet_{}_{}", x, y);
        let handle = open_storage(wallet_name);
        let type_ = CString::new(ITEM_TYPE).unwrap();
        let id = format!("record_{}_{}", x, y);
        let id = CString::new(id).unwrap();
        let tag_names = CString::new(r##"["~new_plaintext", "tag2"]"##).unwrap();

        let err = api::delete_record_tags(handle, type_.as_ptr(), id.as_ptr(), tag_names.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = api::close_storage(handle);
        assert_eq!(err, ErrorCode::Success);
    }

    ///
    /// `Delete Record` driver function.
    ///
    /// Prepares all of the input parameters and calls the corresponding storage plugin api method.
    ///
    /// # Params
    ///  * x: u64 - ID of the calling thread (goes from 0 to THREAD_CNT)
    ///  * y: u64 - ID of the operation being performed by a thread (goes from 0 to OPERATION_COUNT)
    ///
    fn delete_record(x: u64, y: u64) {
        let wallet_name = format!("wallet_{}_{}", x, y);
        let handle = open_storage(wallet_name);

        let id = format!("record_{}_{}", x, y);
        let id = CString::new(id).unwrap();
        let type_ =CString::new(ITEM_TYPE).unwrap();

        let err = api::delete_record(handle, type_.as_ptr(), id.as_ptr());
        assert_eq!(err, ErrorCode::Success);

        let err = api::close_storage(handle);
        assert_eq!(err, ErrorCode::Success);
    }

    ///
    /// `Delete Wallet` driver function.
    ///
    /// Prepares all of the input parameters and calls the corresponding storage plugin api method.
    ///
    /// # Params
    ///  * x: u64 - ID of the calling thread (goes from 0 to THREAD_CNT)
    ///  * y: u64 - ID of the operation being performed by a thread (goes from 0 to OPERATION_COUNT)
    ///
    fn delete_wallet(x: u64, y: u64) {
        let wallet_name = format!("wallet_{}_{}", x, y);
        let wallet_name = CString::new(wallet_name).unwrap();
        let config = CString::new(TEST_CONFIG.get_config()).unwrap();
        let credentials = CString::new(TEST_CONFIG.get_credentials()).unwrap();

        let err = api::delete_storage(wallet_name.as_ptr(), config.as_ptr(), credentials.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    ///
    /// Benchmarking function.
    ///
    /// Accepts a function as an argument, and runs it defined number of times (OPERATIONS_CNT)
    /// via defined number of threads (THREAD_CNT).
    ///
    /// # Params
    ///  * Fn(u64, u64) + Send + Sync + Copy + 'static - function that will be benchmarked
    ///
     fn bench<F>(action: F) where F: Fn(u64, u64) + Send + Sync + Copy + 'static {

        let start_time = Instant::now();

        let mut thread_handles = Vec::new();

        for x in 0..THREAD_CNT {

            let thread = thread::spawn(move || {
                let mut execution_times = Vec::new();

                for y in 0..OPERATIONS_CNT {
                    let time = Instant::now();
                    action(x, y);
                    let time_diff = time.elapsed();
                    execution_times.push(time_diff);
                }

                execution_times
            });

            thread_handles.push(thread);
        }

        let mut all_execution_times = Vec::new();
        for thread in thread_handles {
            all_execution_times.push(thread.join().unwrap());
        }

        let total_execution_time = start_time.elapsed();
        let mut total_execution_time_in_secs = total_execution_time.as_secs();
        if total_execution_time_in_secs == 0 {
            total_execution_time_in_secs = 1;
        }

        let mut max_execution_time = Duration::from_secs(0);
        let mut sum_execution_time = Duration::from_secs(0);
        for time_diffs in all_execution_times {
            max_execution_time = time_diffs.iter().fold(max_execution_time, |acc, cur| max(acc, *cur));
            sum_execution_time = time_diffs.iter().fold(sum_execution_time, |acc, cur| acc + *cur);
        }

        println!("================= Summary =================\n\
                     Max Execution Time:      \t{:?}\n\
                     Operations Executed:     \t{:?}\n\
                     Sum of Exection times:   \t{:?}\n\
                     Total Duration:          \t{:?}\n\
                     Aprox TPS:               \t{:?}",
                 max_execution_time, THREAD_CNT * OPERATIONS_CNT, sum_execution_time, total_execution_time, ((OPERATIONS_CNT * THREAD_CNT) / total_execution_time_in_secs)
        );
    }

    ///
    /// Test Entry point.
    ///
    /// Defines all of the driver functions that need to be passed through the `bench()` function.
    ///
    #[test]
    fn perf_runner() {

        println!("Starting perf runner...");

        let mut actions: Vec<(&str, &(Fn(u64, u64) + Send + Sync + 'static))> = Vec::new();
        actions.push(("Create Wallet", &create_wallet));
        actions.push(("Set Metadata", &set_metadata));
        actions.push(("Get Metadata", &get_metadata));
        actions.push(("Add Record", &add_record));
        actions.push(("Get Record", &get_record));
        actions.push(("Update Record Value", &update_record_value));
        actions.push(("Add Record Tags", &add_record_tags));
        actions.push(("Update Record Tags", &update_record_tags));
        actions.push(("Search Records", &search_records));
        actions.push(("Search All Records", &search_all_records));
        actions.push(("Delete Record Tags", &delete_record_tags));
        actions.push(("Delete Record", &delete_record));
        actions.push(("Delete Wallet", &delete_wallet));

        for (action_name, action) in actions {
            println!("\tBenchmarking {}", action_name);
            bench(action);
        }

        println!("Done!");
    }
}

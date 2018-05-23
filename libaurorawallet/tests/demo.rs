// Local dependencies
extern crate libaurorawallet;

use libaurorawallet::api as api;
use libaurorawallet::errors::error_code::ErrorCode;

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
use std::ffi::{CStr, CString};
use std::slice;

mod demo {

    use super::*;

    lazy_static! {
        static ref TEST_CONFIG: Config = Config::new(ConfigType::QA);
    }

    const ITEM_TYPE: &'static str = "demo-type";

    fn open_storage(wallet_name: String) -> i32 {
        let name = CString::new(wallet_name).unwrap();
        let config = CString::new(TEST_CONFIG.get_config()).unwrap();
        let runtime_config = CString::new(TEST_CONFIG.get_runtime_config()).unwrap();
        let credentials = CString::new(TEST_CONFIG.get_credentials()).unwrap();
        let mut handle: i32 = -1;

        let err = api::open(name.as_ptr(), config.as_ptr(), runtime_config.as_ptr(), credentials.as_ptr(), &mut handle);

        assert_eq!(err, ErrorCode::Success);
        handle
    }

////
////    /** CREATE WALLET */
    fn create_wallet(x: u64, y: u64) {
        let name = format!("wallet_{}_{}", x, y);
        let name = CString::new(name).unwrap();

        let config = CString::new(TEST_CONFIG.get_config()).unwrap();
        let credentials = CString::new(TEST_CONFIG.get_credentials()).unwrap();
        let metadata = CString::new("metadata").unwrap();

        let err = api::create(name.as_ptr(), config.as_ptr(), credentials.as_ptr(), metadata.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    /** ADD RECORD */
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
    }


    /** ADD RECORD */
    fn add_record_1(x: u64, y: u64) {
        let wallet_name = format!("wallet_{}_{}", x, y);
        let handle = open_storage(wallet_name);
        let id = format!("record_{}_{}", x, y);
        let id = CString::new(id).unwrap();
        let type_ =CString::new(ITEM_TYPE).unwrap();
        let value = vec![1, 2, 3, 4];
        let tags_json = CString::new(r##"{"tag1": "value1", "tag2": "value2", "tag3": "value3", "~tag4": "value4",  "~tag5": "value5", "~tag6": "value6"}"##).unwrap();
//        let tags_json = CString::new(r##"{}"##).unwrap();

        let err = api::add_record_1(handle, type_.as_ptr(), id.as_ptr(), value.as_ptr(), value.len(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    /** GET RECORD */
    fn get_record(x: u64, y: u64) {
        let wallet_name = format!("wallet_{}_{}", x, y);
        let handle = open_storage(wallet_name);
        let type_ = CString::new(ITEM_TYPE).unwrap();
        let id = format!("record_{}_{}", x, y);
        let id = CString::new(id).unwrap();
        let options_json = CString::new(r##"{"fetch_value": true, "fetch_tags": true}"##).unwrap();

        let mut record_handle = -1;
        let mut id_p: *const c_char = ptr::null_mut();
        let mut value_p: *const u8 = ptr::null_mut();
        let mut value_len_p = 0;
        let mut tags_json_p: *const c_char = ptr::null_mut();

        let err = api::get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let mut string = String::new();

        let err = api::get_record_id(handle, record_handle, &mut id_p);
        assert_eq!(err, ErrorCode::Success);
        let record_id = unsafe { CStr::from_ptr(id_p) }.to_owned();

        let err = api::get_record_value(handle, record_handle, &mut value_p, &mut value_len_p);
        assert_eq!(err, ErrorCode::Success);
        let record_value = unsafe { slice::from_raw_parts(value_p, value_len_p) };

        let err = api::get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::Success);
        let record_tags = unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap();
    }

    /** GET RECORD */
    fn get_record_1(x: u64, y: u64) {
        let wallet_name = format!("wallet_{}_{}", x, y);
        let handle = open_storage(wallet_name);
        let type_ = CString::new(ITEM_TYPE).unwrap();
        let id = format!("record_{}_{}", x, y);
        let id = CString::new(id).unwrap();
        let options_json = CString::new(r##"{"fetch_value": true, "fetch_tags": true}"##).unwrap();

        let mut record_handle = -1;
        let mut id_p: *const c_char = ptr::null_mut();
        let mut value_p: *const u8 = ptr::null_mut();
        let mut value_len_p = 0;
        let mut tags_json_p: *const c_char = ptr::null_mut();

        let err = api::get_record_1(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
        assert_eq!(err, ErrorCode::Success);

        let mut string = String::new();

        let err = api::get_record_id(handle, record_handle, &mut id_p);
        assert_eq!(err, ErrorCode::Success);
        let record_id = unsafe { CStr::from_ptr(id_p) }.to_owned();

        let err = api::get_record_value(handle, record_handle, &mut value_p, &mut value_len_p);
        assert_eq!(err, ErrorCode::Success);
        let record_value = unsafe { slice::from_raw_parts(value_p, value_len_p) };

        let err = api::get_record_tags(handle, record_handle, &mut tags_json_p);
        assert_eq!(err, ErrorCode::Success);
        let record_tags = unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap();
    }


//    /** UPDATE VALUE */
//    fn update_record_value() {
//        let handle = open_storage();
//
//        let type_ = CString::new(ITEM_TYPE).unwrap();
//        let id = CString::new(ITEM_NAME).unwrap();
//        let new_value = vec![2, 5, 8, 13];
//
//        let err = api::update_record_value(handle, type_.as_ptr(), id.as_ptr(), new_value.as_ptr(), new_value.len());
//        assert_eq!(err, ErrorCode::Success);
//    }

    /** ADD TAGS */
    fn add_record_tags(x: u64, y: u64) {
        let wallet_name = format!("wallet_{}_{}", x, y);
        let handle = open_storage(wallet_name);
        let type_ = CString::new(ITEM_TYPE).unwrap();
        let id = format!("record_{}_{}", x, y);
        let id = CString::new(id).unwrap();
        let tags_json = CString::new(r##"{"new_encrypted": "ASDOPASFO==", "~new_plaintext": "as plain as it can be"}"##).unwrap();
        let err = api::add_record_tags(handle, type_.as_ptr(), id.as_ptr(), tags_json.as_ptr());

        assert_eq!(err, ErrorCode::Success);
    }

    /** ADD TAGS */
    fn add_record_tags_1(x: u64, y: u64) {
        let wallet_name = format!("wallet_{}_{}", x, y);
        let handle = open_storage(wallet_name);
        let type_ = CString::new(ITEM_TYPE).unwrap();
        let id = format!("record_{}_{}", x, y);
        let id = CString::new(id).unwrap();
        let tags_json = CString::new(r##"{"tag1": "new_value1", "new_encrypted": "ASDOPASFO==", "~new_plaintext": "as plain as it can be", "~age": 30}"##).unwrap();
        let err = api::add_record_tags_1(handle, type_.as_ptr(), id.as_ptr(), tags_json.as_ptr());

        assert_eq!(err, ErrorCode::Success);
    }

    /** ADD TAGS */
    fn add_record_tags_2(x: u64, y: u64) {
        let wallet_name = format!("wallet_{}_{}", x, y);
        let handle = open_storage(wallet_name);
        let type_ = CString::new(ITEM_TYPE).unwrap();
        let id = format!("record_{}_{}", x, y);
        let id = CString::new(id).unwrap();
        let tags_json = CString::new(r##"{"tag1": "new_value1", "new_encrypted": "ASDOPASFO==", "~new_plaintext": "as plain as it can be", "~age": 30}"##).unwrap();
        let err = api::add_record_tags_2(handle, type_.as_ptr(), id.as_ptr(), tags_json.as_ptr());

        assert_eq!(err, ErrorCode::Success);
    }

    /** UPDATE TAGS*/
    fn update_record_tags(x: u64, y: u64) {
        let wallet_name = format!("wallet_{}_{}", x, y);
        let handle = open_storage(wallet_name);
        let type_ = CString::new(ITEM_TYPE).unwrap();
        let id = format!("record_{}_{}", x, y);
        let id = CString::new(id).unwrap();
        let tags_json = CString::new(r##"{"new_encrypted": "After update", "~new_plaintext": "After Update"}"##).unwrap();
        let err = api::update_record_tags(handle, type_.as_ptr(), id.as_ptr(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    /** UPDATE TAGS*/
    fn update_record_tags_1(x: u64, y: u64) {
        let wallet_name = format!("wallet_{}_{}", x, y);
        let handle = open_storage(wallet_name);
        let type_ = CString::new(ITEM_TYPE).unwrap();
        let id = format!("record_{}_{}", x, y);
        let id = CString::new(id).unwrap();
        let tags_json = CString::new(r##"{"new_encrypted": "After update", "~new_plaintext": "After Update"}"##).unwrap();
        let err = api::update_record_tags_1(handle, type_.as_ptr(), id.as_ptr(), tags_json.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }


    /** DELETE TAGS */
    fn delete_record_tags(x: u64, y: u64) {
        let wallet_name = format!("wallet_{}_{}", x, y);
        let handle = open_storage(wallet_name);
        let type_ = CString::new(ITEM_TYPE).unwrap();
        let id = format!("record_{}_{}", x, y);
        let id = CString::new(id).unwrap();
        let tag_names = CString::new(r##"["~new_plaintext", "tag2"]"##).unwrap();
        let err = api::delete_record_tags(handle, type_.as_ptr(), id.as_ptr(), tag_names.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }

    /** DELETE TAGS */
    fn delete_record_tags_1(x: u64, y: u64) {
        let wallet_name = format!("wallet_{}_{}", x, y);
        let handle = open_storage(wallet_name);
        let type_ = CString::new(ITEM_TYPE).unwrap();
        let id = format!("record_{}_{}", x, y);
        let id = CString::new(id).unwrap();
        let tag_names = CString::new(r##"["~new_plaintext", "tag2"]"##).unwrap();
        let err = api::delete_record_tags_1(handle, type_.as_ptr(), id.as_ptr(), tag_names.as_ptr());
        assert_eq!(err, ErrorCode::Success);
    }
////
////
////    /** DELETE RECORD */
////    fn delete_record() {
////        let handle = open_storage();
////
////        let id = CString::new(ITEM_NAME).unwrap();
////        let type_ =CString::new(ITEM_TYPE).unwrap();
////
////        let err = api::delete_record(handle, type_.as_ptr(), id.as_ptr());
////        assert_eq!(err, ErrorCode::Success);
////    }
////
////    /** DELETE WALLET */
////    fn delete_wallet() {
////        let name = CString::new(WALLET_NAME).unwrap();
////        let config = CString::new(TEST_CONFIG.get_config()).unwrap();
////        let credentials = CString::new(TEST_CONFIG.get_credentials()).unwrap();
////
////        let err = api::delete(name.as_ptr(), config.as_ptr(), credentials.as_ptr());
////        assert_eq!(err, ErrorCode::Success);
////    }
////
//    fn search_records() {
//        let handle = open_storage();
//
//        let id = CString::new(ITEM_NAME).unwrap();
//        let type_ = CString::new(ITEM_TYPE).unwrap();
//        let mut record_handle = -1;
//        let mut id_p: *const c_char = ptr::null_mut();
//        let mut value_p: *const u8 = ptr::null_mut();
//        let mut value_len_p = 0;
//        let mut tags_json_p: *const c_char = ptr::null_mut();
//
//
//        let query_json = json!({
//            "new_encrypted": {"$in": ["After update", "value2"]},
//            "~new_plaintext": {"$in": ["After update", "value2"]}
//        });
//
////        let query_json = json!({
////            "k1": "v1",
////            "$or": [
////                {
////                    "~k2": {"$like": "like_target"},
////                    "~k3": {"$gte": "100"},
////                    "$not": {
////                        "k4": "v4",
////                        "~k5": {
////                            "$regex": "regex_string"
////                        },
////                    },
////                },
////                {
////                    "k6": {"$in": ["in_string_1", "in_string_2"]},
////                }
////            ],
////            "$not": {
////                "$not": {
////                    "$not": {
////                        "$not": {
////                            "k7": "v7"
////                        }
////                    }
////                }
////            },
////            "$not": {
////                "k8": "v8"
////            }
////        });
//
//        let query_json = serde_json::to_string(&query_json).unwrap();
//
//        let query_json = CString::new(query_json).unwrap();
//
//        let options_json = CString::new(r##"{"fetch_value": true, "fetch_tags": true}"##).unwrap();
//
//        let mut search_handle: i32 = -1;
//
//        println!("HANDLE: {}", handle);
//
////        let err = api::search_records(handle, type_.as_ptr(), query_json.as_ptr(), options_json.as_ptr(), &mut search_handle);
////        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::search_all_records(handle, &mut search_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut total_count: usize = 0;
//
//        let err = api::fetch_search_next_record(handle, search_handle, &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let err = api::get_record(handle, type_.as_ptr(), id.as_ptr(), options_json.as_ptr(), &mut record_handle);
//        assert_eq!(err, ErrorCode::Success);
//
//        let mut string = String::new();
//
//        let err = api::get_record_id(handle, record_handle, &mut id_p);
//        assert_eq!(err, ErrorCode::Success);
//        let record_id = unsafe { CStr::from_ptr(id_p) }.to_owned();
//        println!("\t\tRecord ID: {:?}", record_id);
//
//        let err = api::get_record_value(handle, record_handle, &mut value_p, &mut value_len_p);
//        assert_eq!(err, ErrorCode::Success);
//        let record_value = unsafe { slice::from_raw_parts(value_p, value_len_p) };
//        println!("\t\tRecord Value: {:?}", record_value);
//
//        let err = api::get_record_tags(handle, record_handle, &mut tags_json_p);
//        assert_eq!(err, ErrorCode::Success);
//        let record_tags = unsafe { CStr::from_ptr(tags_json_p) }.to_str().unwrap();
//        println!("\t\tRecord Tags: {:?}", record_tags);
//    }
//
//    #[test]
//    fn runner() {
//        let mut string = String::new();
//
//        println!("Start Demo? ");
//        std::io::stdin().read_line(&mut string).unwrap();
//
//        println!("\nCreate Wallet? ");
//        std::io::stdin().read_line(&mut string).unwrap();
//        create_wallet();
//        println!("\tWallet Created");
//
//        println!("\nAdd Record? ");
//        std::io::stdin().read_line(&mut string).unwrap();
//        add_record();
//        println!("\tRecord Added!");
//
//        println!("\nAdd Tags? ");
//        std::io::stdin().read_line(&mut string).unwrap();
//        add_record_tags();
//        println!("\tTags Added!");
//
//        println!("\nUpdate Tags? ");
//        std::io::stdin().read_line(&mut string).unwrap();
//        update_record_tags();
//        println!("\tTags Updated!");
//
//        println!("\nSearch Record? ");
//        std::io::stdin().read_line(&mut string).unwrap();
//        search_records();
//        println!("\nRecord Found!!!!");
//
//        println!("\nFetch Record? ");
//        std::io::stdin().read_line(&mut string).unwrap();
//        get_record();
//        println!("\tRecord Fetched!");
//
//        println!("\nUpdate Record? ");
//        std::io::stdin().read_line(&mut string).unwrap();
//        update_record_value();
//        println!("\tRecord Updated!");
//
//        println!("\nDelete Tags? ");
//        std::io::stdin().read_line(&mut string).unwrap();
//        delete_record_tags();
//        println!("\tTags Deleted!");
//
//        println!("\nDelete Record? ");
//        std::io::stdin().read_line(&mut string).unwrap();
//        delete_record();
//        println!("\tRecord Deleted");
//
//        println!("\nDelete Wallet? ");
//        std::io::stdin().read_line(&mut string).unwrap();
//        delete_wallet();
//        println!("\tWallet Deleted");
//
//        println!("\nDemo Finished.");
//    }

    use std::cmp::max;
    use std::thread;
    use std::time::{Duration, SystemTime};

    extern crate mysql;


     fn perf_runner<F>(action: F) where F: Fn(u64, u64) + Send + Sync + Copy + 'static {

        const AGENT_CNT: u64 = 1;
        const OPERATIONS_CNT: u64 = 5000;

        let start_time = SystemTime::now();

        let mut results = Vec::new();

        for x in 0..AGENT_CNT {

            let thread = thread::spawn(move || {
                let mut time_diffs = Vec::new();

                for y in 0..OPERATIONS_CNT {
                    let time = SystemTime::now();
                    action(x, y);
                    let time_diff = SystemTime::now().duration_since(time).unwrap();
                    time_diffs.push(time_diff);
                }

                time_diffs
            });

            results.push(thread);
        }

        let mut all_diffs = Vec::new();
        for result in results {
            all_diffs.push(result.join().unwrap());
        }
        let total_duration = SystemTime::now().duration_since(start_time).unwrap();


        let mut time_diff_max = Duration::from_secs(0);
        let mut time_sum_diff = Duration::from_secs(0);
        for time_diffs in all_diffs {
            time_diff_max = time_diffs.iter().fold(time_diff_max, |acc, cur| max(acc, *cur));
            time_sum_diff = time_diffs.iter().fold(time_sum_diff, |acc, cur| acc + *cur);
        }

        println!("================= Summary =================\n\
                     Max Execution Time:      \t{:?}\n\
                     Operations Executed:     \t{:?}\n\
                     Sum of Exection times:   \t{:?}\n\
                     Total Duration:          \t{:?}\n\
                     Aprox TPS:               \t{:?}",
            time_diff_max, AGENT_CNT * OPERATIONS_CNT, time_sum_diff, total_duration, ((OPERATIONS_CNT * AGENT_CNT) / total_duration.as_secs())
        );
    }




    #[test]
    fn benchmark() {

        const AGENT_CNT: u64 = 1;
        const OPERATIONS_CNT: u64 = 5000;

        let start_time = SystemTime::now();

        let mut results = Vec::new();

        for x in 0..AGENT_CNT {

            let thread = thread::spawn(move || {
                let mut time_diffs = Vec::new();

                for y in 0..OPERATIONS_CNT {
                    let time = SystemTime::now();
//                    create_wallet(x, y);
//                    add_record(x, y);
//                    add_record_1(x, y);
//                    get_record(x, y);
//                    get_record_1(x, y);
//                    add_record_tags(x, y);
//                    add_record_tags_1(x, y);
//                    add_record_tags_2(x, y);
//                    update_record_tags(x, y);
//                    update_record_tags_1(x, y);
//                    delete_record_tags(x, y);
                    delete_record_tags_1(x, y);
                    let time_diff = SystemTime::now().duration_since(time).unwrap();
                    time_diffs.push(time_diff);
                }

                time_diffs
            });

            results.push(thread);
        }

        let mut all_diffs = Vec::new();
        for result in results {
            all_diffs.push(result.join().unwrap());
        }
        let total_duration = SystemTime::now().duration_since(start_time).unwrap();


        let mut time_diff_max = Duration::from_secs(0);
        let mut time_sum_diff = Duration::from_secs(0);
        for time_diffs in all_diffs {
            time_diff_max = time_diffs.iter().fold(time_diff_max, |acc, cur| max(acc, *cur));
            time_sum_diff = time_diffs.iter().fold(time_sum_diff, |acc, cur| acc + *cur);
        }

        println!("================= Summary =================\n\
                     Max Execution Time:      \t{:?}\n\
                     Operations Executed:     \t{:?}\n\
                     Sum of Exection times:   \t{:?}\n\
                     Total Duration:          \t{:?}\n\
                     Aprox TPS:               \t{:?}",
            time_diff_max, AGENT_CNT * OPERATIONS_CNT, time_sum_diff, total_duration, ((OPERATIONS_CNT * AGENT_CNT) / total_duration.as_secs())
        );
    }

    #[test]
    fn testoo() {

        println!("Starting perf runner...");

        let mut actions: Vec<(&str, &(Fn(u64, u64) + Send + Sync + 'static))> = Vec::new();
        actions.push(("Create Wallet", &create_wallet));
        actions.push(("Add Record", &add_record));
        actions.push(("Add Record 1", &add_record_1));
        actions.push(("Get Record", &get_record));
        actions.push(("Get Record 1", &get_record_1));
        actions.push(("Add Record Tags", &add_record_tags));
        actions.push(("Add Record Tags 1", &add_record_tags_2));
        actions.push(("Update Record Tags", &update_record_tags));
        actions.push(("Update Record Tags 1", &update_record_tags_1));
        actions.push(("Delete Record Tags", &delete_record_tags));
        actions.push(("Delete Record Tags", &delete_record_tags_1));

        for (action_name, action) in actions {
            println!("\tBenchmarking {}", action_name);
            perf_runner(action);
            println!("\t{} Done", action_name);
        }

        println!("Done!");
    }
}




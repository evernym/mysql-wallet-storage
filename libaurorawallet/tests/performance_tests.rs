extern crate aurorastorage;

use aurorastorage::utils::multi_pool::{StorageConfig, StorageCredentials};
use std::cmp::max;
use std::thread;
use std::time::{Duration, SystemTime};

pub mod test_utils;
use test_utils::api_requests::api_requests;
use test_utils::helper_functions::{get_random_record_value, get_hash_map_from_json_string,
                                   random_string};
use test_utils::config::{Config, ConfigType};
use std::collections::HashMap;

extern crate mysql;
use mysql::Conn;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate lazy_static;
extern crate core;

/// `DB_THREADS_CNT` - number of threads used for db population
/// should be less or equal to max db connections supported on DB server
/// TOTAL_WALLET_CNT % DB_THREADS_CNT should be 0
const DB_THREADS_CNT: u64 = 25;

const THREADS_CNT: u64 = 1;
const TOTAL_WALLET_CNT: u64 = 10000;
const RECORDS_PER_WALLET_CNT: u64 = 10;

    ///
    /// Populates DB with data needed for tests execution
    /// Number of threads used to populate DB is defined in constant THREADS_CNT.
    ///
    ///
    /// # Arguments
    ///
    ///  * `wallet_cnt` - total wallet count
    ///  * `records_per_wallet_cnt` - number of records per wallet (can be 0)
    ///  * `custom_tags_per_record_data` - json string representing tags or search query (can be "")
    ///  * `percent_of_custom_tags_per_record` - int representing percent of tags per
    ///     record that will have same values as provided. Other records will have same tags
    ///     but with random chosen values.
    ///
    fn populate_database(wallet_cnt: u64, records_per_wallet_cnt: u64, custom_tags_per_record_data: &'static str, percent_of_custom_tags_per_record: u64) {
        println!("Start populating DB....");
        let mut results = Vec::new();

        for thread_num in 1..DB_THREADS_CNT +1 {

            let thread = thread::spawn(move || {
                let record_value = get_random_record_value();

                for wallet_num in (thread_num - 1)*(wallet_cnt/ DB_THREADS_CNT)+1..thread_num*(wallet_cnt/ DB_THREADS_CNT)+1 {
                    let wallet_name = format!("wallet_name_{}", wallet_num);
                    api_requests::create_wallet(&wallet_name);
                    let handle: i32 = api_requests::open_storage(&wallet_name);
                    if records_per_wallet_cnt != 0 {
                        for i in 1..records_per_wallet_cnt + 1 {
                            let record_id = format!("record_id_{}_{}", wallet_num, i);
                            let mut tags_list: HashMap<String, String> = HashMap::new();
                            let mut custom_tags: HashMap<String, String> = HashMap::new();
                            if custom_tags_per_record_data != "" && percent_of_custom_tags_per_record!=0 {
                                custom_tags = get_hash_map_from_json_string(custom_tags_per_record_data);
                                let num_of_records_wih_custom_tags = (records_per_wallet_cnt * percent_of_custom_tags_per_record)/100;
                                for (tag_name, tag_value) in custom_tags {
                                    if i>=1 && i<=num_of_records_wih_custom_tags{
                                        tags_list.insert(tag_name, tag_value);

                                    } else {
                                        tags_list.insert(tag_name, random_string(10));
                                    }
                                }
                            }
                        let tags = serde_json::to_string(&tags_list).unwrap();
                        api_requests::add_record(handle, &record_id, &record_value, &tags);
                        }
                    }
                }
            });
             results.push(thread);
        }
         for result in results {
            result.join().unwrap();
        }
        println!("Finished with populating DB.");
    }

    ///
    /// Enum representing all aurora api requests
    ///
    enum Action {
        AddWallet,
        SetMetadata,
        GetMetadata,
        OpenAndCloseWallet,
        DeleteWallet,
        AddRecord,
        GetRecord,
        DeleteRecord,
        UpdateRecordValue,
        AddRecordTags,
        UpdateRecordTags,
        DeleteRecordTags,
        SearchRecords,
        SearchAllRecords
    }
    ///
    /// Defines steps that should be executed for each aurora wallet api request.
    ///
    ///
    /// # Arguments
    ///
    ///  * `wallet_number` - ordinal number of wallet per thread
    ///  * `records_per_wallet_cnt` - number of records per wallet (can be 0)
    ///  * `custom_tags_per_record_data` - json string representing tags or search query(can be "")
    ///  * `action` - enum that represents which specific api call should be executed
    ///     Can take any value defined in `Action` enum
    ///

    fn execute_action(wallet_number: u64, records_per_wallet_cnt: u64, custom_tags_per_record_data: &'static str, action: &Action, handle: i32){
        let mut record_id;
        match action {
            &Action::AddWallet =>{
                let wallet_name = format!("wallet_name_{}", wallet_number);
                api_requests::create_wallet(&wallet_name);
            },
            &Action::SetMetadata =>{
                let new_metadata: String = format!("Set metadata {}", random_string(20));
                api_requests::set_metadata(handle, &new_metadata);
            },
            &Action::GetMetadata =>{
                api_requests::get_metadata(handle);
            }
            &Action::OpenAndCloseWallet => {
                let wallet_name = format!("wallet_name_{}", wallet_number);
                api_requests::open_and_close_storage(&wallet_name);
            }
            &Action::DeleteWallet =>{
                let wallet_name = format!("wallet_name_{}", wallet_number);
                api_requests::delete_wallet(&wallet_name);
            },
            &Action::AddRecord => {
                let record_value = get_random_record_value();
                    for i in 1..records_per_wallet_cnt + 1 {
                        record_id = format!("record_id_{}_{}", wallet_number, i);
                         let mut tags_list: HashMap<String, String> = HashMap::new();
                            if custom_tags_per_record_data != "" {
                                tags_list = get_hash_map_from_json_string(custom_tags_per_record_data);
                            }
                            let tags = serde_json::to_string(&tags_list).unwrap();
                            api_requests::add_record(handle, &record_id, &record_value, &tags);
                    }
            },
            &Action::GetRecord =>{
                for i in 1..records_per_wallet_cnt+1 {
                    record_id = format!("record_id_{}_{}", wallet_number, i);
                    api_requests::get_record_with_details(handle, &record_id);
                }
            },
            &Action::DeleteRecord =>{
                for i in 1..records_per_wallet_cnt+1 {
                    record_id = format!("record_id_{}_{}", wallet_number, i);
                    api_requests::delete_record(handle, &record_id);
                }
            },
            &Action::UpdateRecordValue =>{
                let new_record_value = get_random_record_value();
                for i in 1..records_per_wallet_cnt + 1 {
                    record_id = format!("record_id_{}_{}", wallet_number, i);
                    api_requests::update_record_value(handle, &record_id, &new_record_value);
                }
            },
            &Action::AddRecordTags =>{
                for i in 1..records_per_wallet_cnt + 1 {
                    record_id = format!("record_id_{}_{}", wallet_number, i);
                    api_requests::add_record_tags(handle, &record_id,  &custom_tags_per_record_data);

                }
            }
            &Action::UpdateRecordTags =>{
                for i in 1..records_per_wallet_cnt + 1 {
                    record_id = format!("record_id_{}_{}", wallet_number, i);
                    api_requests::update_record_tags(handle, &record_id,  &custom_tags_per_record_data);
                }
            },
            &Action::DeleteRecordTags => {
                for i in 1..records_per_wallet_cnt + 1 {
                    record_id = format!("record_id_{}_{}", wallet_number, i);
                    api_requests::delete_record_tags(handle, &record_id,  &custom_tags_per_record_data);
                }
            },
            &Action::SearchRecords =>{
                api_requests::search_records(handle, &custom_tags_per_record_data);
            }
            &Action::SearchAllRecords =>{
                api_requests::search_all_records(handle);
            }

        }
    }

    ///
    /// Sends aurora api requests in parallel and calculate time needed for execution.
    /// Number of threads is defined in constant THREADS_CNT.
    ///
    ///
    /// # Arguments
    ///
    ///  * `wallet_cnt` - total wallet count
    ///  * `records_per_wallet_cnt` - number of records per wallet (can be 0)
    ///  * `custom_tags_per_record_data` - json string representing tags or search query(can be "")
    ///  * `action` - enum that represents which specific api call should be executed
    ///     Can take any value defined in `Action` enum
    ///
    fn send_requests(mut wallet_cnt: u64, mut records_per_wallet_cnt: u64, custom_tags_per_record_data: &'static str, action: &'static Action) {

        let start_time = SystemTime::now();

        let mut thread_handles = Vec::new();

        for thread_num in 1..THREADS_CNT +1 {

            let thread = thread::spawn(move || {
                let mut execution_times = Vec::new();
                for wallet_num  in (thread_num - 1)*(wallet_cnt/ THREADS_CNT)+1..thread_num*(wallet_cnt/ THREADS_CNT)+1{
                    let wallet_name = format!("wallet_name_{}", wallet_num);
                    let handle = match action {
                        &Action::AddWallet | &Action::DeleteWallet | &Action::OpenAndCloseWallet => -1,
                        _ => api_requests::open_storage(&wallet_name)
                    };

                    let time = SystemTime::now();
                    execute_action( wallet_num, records_per_wallet_cnt,  custom_tags_per_record_data, &action, handle);
                    let time_diff = SystemTime::now().duration_since(time).unwrap();
                    execution_times.push(time_diff);
                }
                execution_times
            });

            thread_handles.push(thread);
        }

        let mut all_execution_times = Vec::new();
        for result in thread_handles {
            all_execution_times.push(result.join().unwrap());
        }
        let total_execution_time = SystemTime::now().duration_since(start_time).unwrap();
        let mut total_execution_time_in_secs = total_execution_time.as_secs();
        if total_execution_time_in_secs == 0 {
            total_execution_time_in_secs = 1;
        }
        if wallet_cnt == 0{
            wallet_cnt = 1;
        }
        if records_per_wallet_cnt == 0{
            records_per_wallet_cnt = 1;
        }

        let mut max_execution_time = Duration::from_secs(0);
        let mut sum_execution_time = Duration::from_secs(0);
        for exec_time in all_execution_times {
            max_execution_time = exec_time.iter().fold(max_execution_time, |acc, cur| max(acc, *cur));
            sum_execution_time = exec_time.iter().fold(sum_execution_time, |acc, cur| acc + *cur);
        }

        println!("================= Summary =================\n\
                     Max Execution Time:      \t{:?}\n\
                     Operations Executed:     \t{:?}\n\
                     Sum of Exection times:   \t{:?}\n\
                     Total Duration:          \t{:?}\n\
                     Aprox TPS:               \t{:?}",
                 max_execution_time,  wallet_cnt * records_per_wallet_cnt , sum_execution_time, total_execution_time, ((wallet_cnt * records_per_wallet_cnt) / total_execution_time_in_secs)
        );
}
    ///
    /// Deletes all from `wallets` table
    ///
    fn cleanup(){
        let test_config: Config = Config::new(ConfigType::QA);
        let cred_str = test_config.get_credentials();
        let credentials: StorageCredentials = serde_json::from_str(cred_str.as_ref()).unwrap();
        let config_str = test_config.get_config();
        let config: StorageConfig = serde_json::from_str(config_str.as_ref()).unwrap();
        let connection_string = format!("mysql://{}:{}@{}:{}/{}", credentials.user, credentials.pass, config.read_host, config.port, config.db_name);
        let mut connection = Conn::new(connection_string).unwrap();
        connection.query("delete from wallets").unwrap();
    }

mod performance {
    use super::*;
    #[test]
    fn test_add_wallet(){
        cleanup();
        send_requests( TOTAL_WALLET_CNT,  0, "",  &Action::AddWallet);
    }

    #[test]
    fn test_delete_wallet(){
        cleanup();
        populate_database(TOTAL_WALLET_CNT, RECORDS_PER_WALLET_CNT, r#"{"name": "John", "surname": "Doe"}"#, 100);
        send_requests( TOTAL_WALLET_CNT, 0,  "", &Action::DeleteWallet);
    }

    #[test]
    fn test_set_metadata(){
        cleanup();
        populate_database(TOTAL_WALLET_CNT, 0, "", 0);
        send_requests(TOTAL_WALLET_CNT, 0, "", &Action::SetMetadata);
    }

    #[test]
    fn test_get_metadata(){
        cleanup();
        populate_database(TOTAL_WALLET_CNT, 0, "", 0);
        send_requests(TOTAL_WALLET_CNT, 0, "", &Action::GetMetadata);
    }

    #[test]
    fn test_open_and_close_wallet(){
        cleanup();
        populate_database(TOTAL_WALLET_CNT, 0, "", 0);
        send_requests(TOTAL_WALLET_CNT, 0, "", &Action::OpenAndCloseWallet);
    }

    #[test]
    fn test_add_record_with_tags(){
        cleanup();
        populate_database(TOTAL_WALLET_CNT, 0,  "", 0);
        send_requests( TOTAL_WALLET_CNT, RECORDS_PER_WALLET_CNT, r#"{"name": "John", "surname": "Doe"}"#, &Action::AddRecord);
    }

    #[test]
    fn test_get_record(){
        cleanup();
        populate_database(TOTAL_WALLET_CNT, RECORDS_PER_WALLET_CNT,  r#"{"name": "John", "surname": "Doe"}"#, 100);
        send_requests( TOTAL_WALLET_CNT, RECORDS_PER_WALLET_CNT, "", &Action::GetRecord);
    }

    #[test]
    fn test_delete_record(){
        cleanup();
        populate_database(TOTAL_WALLET_CNT, RECORDS_PER_WALLET_CNT,  r#"{"name": "John", "surname": "Doe"}"#, 100);
        send_requests( TOTAL_WALLET_CNT, RECORDS_PER_WALLET_CNT, "", &Action::DeleteRecord);
    }

    #[test]
    fn test_update_record_value(){
        cleanup();
        populate_database(TOTAL_WALLET_CNT, RECORDS_PER_WALLET_CNT,  r#"{"name": "John", "surname": "Doe"}"#, 100);
        send_requests( TOTAL_WALLET_CNT, RECORDS_PER_WALLET_CNT, "", &Action::UpdateRecordValue);
    }

    #[test]
    fn test_add_record_tags(){
        cleanup();
        populate_database(TOTAL_WALLET_CNT, RECORDS_PER_WALLET_CNT,  r#"{"tag1": "value1", "tag2": "value2", "tag3": "value3"}"#, 100);
        send_requests( TOTAL_WALLET_CNT, RECORDS_PER_WALLET_CNT,  r#"{"tag1": "newValue1", "name": "John", "surname": "Doe"}"#,&Action::AddRecordTags);
    }

    #[test]
    fn test_update_record_tags(){
        cleanup();
        populate_database(TOTAL_WALLET_CNT, RECORDS_PER_WALLET_CNT,  r#"{"name": "John", "surname": "Doe"}"#, 100);
        send_requests( TOTAL_WALLET_CNT, RECORDS_PER_WALLET_CNT,  r#"{"name": "UpdatedName", "surname": "UpdatedSurname"}"#, &Action::UpdateRecordTags);

    }

    #[test]
    fn test_delete_record_tags(){
        cleanup();
        populate_database(TOTAL_WALLET_CNT, RECORDS_PER_WALLET_CNT, r#"{"name": "John", "surname": "Doe"}"#, 100);
        send_requests( TOTAL_WALLET_CNT, RECORDS_PER_WALLET_CNT, r#"["name"]"#, &Action::DeleteRecordTags);
    }

    #[test]
    fn test_search_record(){
        cleanup();
        populate_database(TOTAL_WALLET_CNT, RECORDS_PER_WALLET_CNT, r#"{"name": "John", "surname": "Doe", "country": "Serbia"}"#, 20);
        send_requests( TOTAL_WALLET_CNT, 0,  r#"{"name": {"$in": ["John", "john"]}, "age": {"$in": ["Serbia", "serbia"]}}"#,&Action::SearchRecords);
    }

    #[test]
    fn test_search_all_records(){
        cleanup();
        populate_database(TOTAL_WALLET_CNT, RECORDS_PER_WALLET_CNT, r#"{"name": "John", "surname": "Doe"}"#, 100);
        send_requests( TOTAL_WALLET_CNT, 0,  "", &Action::SearchAllRecords);
    }
}

extern crate aurorastorage;

use std::cmp::max;
use std::thread;
use std::time::{Duration, SystemTime};

pub mod test_utils;
use test_utils::api_requests::api_requests;
use test_utils::helper_functions::{get_random_record_value, generate_predefinied_tags, get_predefined_tag_names};
use test_utils::config::{Config, ConfigType};

extern crate mysql;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate lazy_static;
extern crate core;

const THREADS_CNT: u64 = 10;

    ///
    /// Populates DB with data needed for tests execution
    /// Number of threads used to populate DB is defined in constant THREADS_CNT.
    ///
    ///
    /// # Arguments
    ///
    ///  * `wallet_cnt` - wallet count per thread
    ///  * `records_per_wallet_cnt` - number of records per wallet (can be 0)
    ///  * `tags_per_record_cnt` - number of tags per record (can be 0)
    ///
    ///
    fn populate_database(wallet_cnt: u64, record_per_wallet_cnt: u64, tags_per_record_cnt: u64) {
        println!("Start populating DB....");
        let mut results = Vec::new();

        for a in 1..THREADS_CNT+1 {

            let thread = thread::spawn(move || {
                let record_value = get_random_record_value();

                for w in 1..wallet_cnt/THREADS_CNT+1 {
                    let wallet_name = format!("wallet_name_{}_{}", a, w);
                    api_requests::create_wallet(&wallet_name);

                    if record_per_wallet_cnt != 0 {
                        for i in 1..record_per_wallet_cnt + 1 {
                            let record_id = format!("record_id_{}_{}_{}", a, w, i);
                            let mut tags: String = "{}".to_string();
                            if tags_per_record_cnt != 0 {
                                 tags = generate_predefinied_tags(tags_per_record_cnt);
                            }
                                api_requests::add_record(&wallet_name, &record_id, &record_value, &tags);
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
        DeleteWallet,
        AddRecord,
        GetRecord,
        DeleteRecord,
        UpdateRecordValue,
        AddRecordTags,
        UpdateRecordTags,
        DeleteRecordTags

    }
    ///
    /// Defines steps that should be executed for each aurora wallet api request.
    ///
    ///
    /// # Arguments
    ///
    ///  * `action` - enum that represents which specific api call should be executed
    ///     Can take any value defined in `Action` enum
    ///  * `wallet_cnt` - wallet count per thread
    ///  * `records_per_wallet_cnt` - number of records per wallet (can be 0)
    ///  * `tags_per_record_cnt` - number of tags per record (can be 0)
    ///
    ///

    fn execute_action(action: &Action, thread_number: u64, wallet_number_per_thread: u64, records_per_wallet_cnt: u64, tags_per_record_cnt: u64){
        let wallet_name = format!("wallet_name_{}_{}", thread_number, wallet_number_per_thread);
        let mut record_id;
        match action {
            &Action::AddWallet =>{
                api_requests::create_wallet(&wallet_name);
            },
            &Action::DeleteWallet =>{
                api_requests::delete_wallet(&wallet_name);
            }
            &Action::AddRecord => {
                let record_value = get_random_record_value();
                    for i in 1..records_per_wallet_cnt + 1 {
                        record_id = format!("record_id_{}_{}_{}", thread_number, wallet_number_per_thread, i);
                        let mut tags: String = "{}".to_string();
                        if tags_per_record_cnt != 0 {
                             tags = generate_predefinied_tags(tags_per_record_cnt);
                        }
                        api_requests::add_record(&wallet_name, &record_id, &record_value, &tags);
                    }
            },
            &Action::GetRecord =>{
                for i in 1..records_per_wallet_cnt+1 {
                    record_id = format!("record_id_{}_{}_{}", thread_number, wallet_number_per_thread, i);
                    api_requests::get_record_with_details(&wallet_name, &record_id);
                }
            },
            &Action::DeleteRecord =>{
                for i in 1..records_per_wallet_cnt+1 {
                    record_id = format!("record_id_{}_{}_{}", thread_number, wallet_number_per_thread, i);
                    api_requests::delete_record(&wallet_name, &record_id);
                }
            },
            &Action::UpdateRecordValue =>{
                let new_record_value = get_random_record_value();
                for i in 1..records_per_wallet_cnt + 1 {
                    record_id = format!("record_id_{}_{}_{}", thread_number, wallet_number_per_thread, i);
                    api_requests::update_record_value(&wallet_name, &record_id, &new_record_value);
                }
            },
            &Action::AddRecordTags =>{
                    for i in 1..records_per_wallet_cnt + 1 {
                        record_id = format!("record_id_{}_{}_{}", thread_number, wallet_number_per_thread, i);
                        if tags_per_record_cnt != 0 {
                            let mut tags: String = generate_predefinied_tags(tags_per_record_cnt);
                            api_requests::add_record_tags(&wallet_name, &record_id,  &tags);
                        }
                    }
            }
            &Action::UpdateRecordTags =>{
                    for i in 1..records_per_wallet_cnt + 1 {
                        record_id = format!("record_id_{}_{}_{}", thread_number, wallet_number_per_thread, i);
                        if tags_per_record_cnt != 0 {
                            let mut tags: String = generate_predefinied_tags(tags_per_record_cnt);
                            api_requests::update_record_tags(&wallet_name, &record_id,  &tags);
                        }
                    }
            },
            &Action::DeleteRecordTags => {
                    for i in 1..records_per_wallet_cnt + 1 {
                        record_id = format!("record_id_{}_{}_{}", thread_number, wallet_number_per_thread, i);
                        if tags_per_record_cnt != 0 {
                            let mut tags: Vec<String> = get_predefined_tag_names(tags_per_record_cnt);
                            api_requests::delete_record_tags(&wallet_name, &record_id,  &tags);
                        }
                    }
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
    ///  * `wallet_cnt` - wallet count per thread
    ///  * `records_per_wallet_cnt` - number of records per wallet (can be 0)
    ///  * `tags_per_record_cnt` - number of tags per record (can be 0)
    ///  * `action` - enum that represents which specific api call should be executed
    ///     Can take any value defined in `Action` enum
    ///
    fn send_requests(mut wallet_cnt: u64, mut records_per_wallet_cnt: u64, mut tags_per_record_cnt: u64, action: &'static Action) {

        let start_time = SystemTime::now();

        let mut thread_handles = Vec::new();

        for a in 1..THREADS_CNT+1 {

            let thread = thread::spawn(move || {
                let mut execution_times = Vec::new();
                let time = SystemTime::now();
                for w  in 1..(wallet_cnt/THREADS_CNT)+1{
                    let time = SystemTime::now();
                    execute_action(&action, a, w, records_per_wallet_cnt, tags_per_record_cnt);
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

        if tags_per_record_cnt == 0{
            tags_per_record_cnt = 1;
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
                 max_execution_time, THREADS_CNT * wallet_cnt * records_per_wallet_cnt * tags_per_record_cnt, sum_execution_time, total_execution_time, ((THREADS_CNT * wallet_cnt * records_per_wallet_cnt * tags_per_record_cnt) / total_execution_time_in_secs)
        );
}

    fn cleanup(){
        //delete all from wallet table
    }

mod performance {
    use super::*;

    #[test]
    fn test_add_wallet(){
        cleanup();
        send_requests( 100, 0, 0, &Action::AddWallet);
    }

    #[test]
    fn test_delete_wallet(){
        cleanup();
        populate_database(100, 10, 10);
        send_requests( 100, 0, 0, &Action::DeleteWallet);
    }

    #[test]
    fn test_add_record_without_tags(){
        cleanup();
        populate_database(100, 0, 0);
        send_requests( 100, 1, 0, &Action::AddRecord);

    }

    #[test]
    fn test_add_record_with_tags(){
        cleanup();
        populate_database(100, 0, 0);
        send_requests( 100, 6, 3, &Action::AddRecord);

    }

    #[test]
    fn test_get_record(){
        cleanup();
        populate_database(100, 10, 0);
        send_requests( 100, 10, 0, &Action::GetRecord);
    }

    #[test]
    fn test_delete_record(){
        cleanup();
        populate_database(50, 10, 10);
        send_requests( 50, 10, 0, &Action::DeleteRecord);
    }

    #[test]
    fn test_update_record_value(){
        cleanup();
        populate_database(50, 10, 10);
        send_requests( 50, 10, 1, &Action::UpdateRecordValue);
    }

    #[test]
    fn test_add_record_tags(){
        cleanup();
        populate_database(50, 10, 0);
        send_requests( 50, 10, 10, &Action::AddRecordTags);

    }

    #[test]
    fn test_update_record_tags(){
        cleanup();
        populate_database(50, 10, 10);
        send_requests( 50, 10, 10, &Action::UpdateRecordTags);

    }

    #[test]
    fn test_delete_record_tags(){
        cleanup();
        populate_database(50, 10, 10);
        send_requests( 50, 10, 10, &Action::DeleteRecordTags);

    }

}

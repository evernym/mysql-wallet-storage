use utils::handle_store::HandleStore;
use std::sync::Arc;
use mysql::{Pool, Transaction, QueryResult, Error};
use utils::error_code::ErrorCode;
use std::collections::HashMap;
use std::ffi::CString;
use serde_json;

fn default_true() -> bool {
    true
}

#[derive(Deserialize)]
pub struct FetchOptions {
    #[serde(default="default_true")]
    fetch_value: bool,

    #[serde(default="default_true")]
    fetch_tags: bool,
}

pub struct Record {
    pub id: CString,
    pub value: Option<Vec<u8>>,
    pub tags: Option<CString>,
    pub type_: Option<CString>,
}

impl Record {
    fn new(id: CString, value: Option<Vec<u8>>, tags: Option<CString>, type_: Option<CString>) -> Self {
        Self{id, value, tags, type_}
    }
}

struct SearchRecord {
}

pub struct AuroraStorage {
    wallet_id: u64,
    records: HandleStore<Record>,
    searches: HandleStore<SearchRecord>,
    read_pool: Arc<Pool>, // cached reference to the pool
    write_pool: Arc<Pool>,
}

impl AuroraStorage {
    pub fn new(wallet_id: u64, read_pool: Arc<Pool>, write_pool: Arc<Pool>) -> Self {
        Self{wallet_id, records: HandleStore::new(), searches: HandleStore::new(), read_pool, write_pool}
    }

    pub fn free_record(&self, record_handle: i32) -> ErrorCode {
        if self.records.remove(record_handle) {
            ErrorCode::Success
        }
        else {
            ErrorCode::InvalidRecordHandle
        }
    }

    pub fn free_search(&self, search_handle: i32) -> ErrorCode {
        if self.searches.remove(search_handle) {
            ErrorCode::Success
        }
        else {
            ErrorCode::InvalidSearchHandle
        }
    }

    pub fn add_record(&self, type_: &str, id: &str, value: &Vec<u8>, tags: &HashMap<String, String>) -> ErrorCode {
        // start an transaction
        let mut transaction = check_result!(
            self.write_pool.start_transaction(true, None, Some(false)),
            ErrorCode::DatabaseError);

        let item_id = {
            let result: Result<QueryResult, Error> = transaction.prep_exec(
                "INSERT INTO items (type, name, value, wallet_id) VALUE (:type, :name, :value, :wallet_id)",
                params!{
                    "type" => type_,
                    "name" => id,
                    "value" => value,
                    "wallet_id" => self.wallet_id
                }
            );

            let result: QueryResult = match result {
                Err(Error::MySqlError(_)) => return ErrorCode::RecordAlreadExists,
                Err(_) => return ErrorCode::DatabaseError,
                Ok(result) => result,
            };

            result.last_insert_id()
        };

        for (tag_name, tag_value) in tags {
            let first_char = tag_name.chars().next().unwrap_or('\0');
            if first_char == '~' { // plain text
                let mut tag_name = tag_name.clone();
                tag_name.remove(0);
                check_result!(
                    transaction.prep_exec(
                        "INSERT INTO tags_plaintext (name, value, item_id) VALUE (:name, :value, :item_id)",
                        params!{
                            "name" => tag_name,
                            "value" => tag_value,
                            "item_id" => item_id
                        }
                    ),
                    ErrorCode::DatabaseError
                );
            }
            else {
                check_result!(
                    transaction.prep_exec(
                        "INSERT INTO tags_encrypted (name, value, item_id) VALUE (:name, :value, :item_id)",
                        params!{
                            "name" => tag_name,
                            "value" => tag_value,
                            "item_id" => item_id
                        }
                    ),
                    ErrorCode::DatabaseError
                );
            }
        }

        check_result!(transaction.commit(), ErrorCode::DatabaseError);

        ErrorCode::Success
    }

    pub fn fetch_record(&self, type_: &str, id: &str, options: &str, record_handle_p: *mut i32) -> ErrorCode {
        let options: FetchOptions = check_result!(serde_json::from_str(options), ErrorCode::InvalidJSON);

        let query = format!(
            "SELECT i.id, {}, {} \
                FROM items i \
                WHERE wallet_id = :wallet_id \
                    AND type = :type \
                    AND name = :name",
            if options.fetch_value { "value" } else {"''"},
            if options.fetch_tags {
                "CONCAT( \
                    '{', \
                    IFNULL( \
                        concat(\
                            (select group_concat(concat(json_quote(concat('~', name)), ':', json_quote(value))) from tags_plaintext WHERE item_id = i.id), \
                            ','\
                        ), \
                        ''\
                    ), \
                    IFNULL(\
                        (select group_concat(concat(json_quote(name), ':', json_quote(value))) from tags_encrypted WHERE item_id = i.id), \
                        ''\
                    ), \
                    '}' \
                ) tags"
            }
            else {"''"}
        );

        let mut result: QueryResult = check_result!(
            self.read_pool.prep_exec(
                &query,
                params!{
                    "wallet_id" => self.wallet_id,
                    "type" => type_,
                    "name" => id
                }
            ),
            ErrorCode::DatabaseError
        );

        let row = check_result!(check_option!(result.next(), ErrorCode::NoRecord), ErrorCode::DatabaseError);

        let db_id: u64 = row.get(0).unwrap();

        // These 2 value cannot be NULL.
        let db_value: Vec<u8> = check_option!(row.get(1), ErrorCode::DatabaseError);
        let tags: String = check_option!(row.get(2), ErrorCode::DatabaseError);

        let record = Record::new(
            check_result!(CString::new(id), ErrorCode::InvalidEncoding),
            if options.fetch_value {Some(db_value)} else {None},
            if options.fetch_tags {Some(check_result!(CString::new(tags), ErrorCode::InvalidEncoding))} else {None},
            None
        );

        let record_handle = self.records.insert(record);

        unsafe { *record_handle_p = record_handle; }

        ErrorCode::Success
    }

    pub fn get_record(&self, record_handle: i32) -> Option<Arc<Record>> {
        self.records.get(record_handle)
    }

    pub fn delete_record(&self, type_: &str, id: &str) -> ErrorCode {
        let result = check_result!(
            self.write_pool.prep_exec(
                "DELETE FROM items WHERE wallet_id = :wallet_id AND type = :type AND name = :name",
                params! {
                    "wallet_id" => self.wallet_id,
                    "type" => type_,
                    "name" => id
                }
            ),
            ErrorCode::DatabaseError
        );

        if result.affected_rows() != 1 {
            return ErrorCode::UnknownItem;
        }

        ErrorCode::Success
    }
}
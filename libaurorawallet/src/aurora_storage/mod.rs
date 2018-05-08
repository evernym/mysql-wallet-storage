pub mod statement;

use utils::handle_store::HandleStore;
use aurora_storage::statement::Statement;

use std::sync::Arc;
use mysql::{Pool, QueryResult, Error};
use errors::error_code::ErrorCode;
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

    ///
    /// Removes a record handle, thus removing the referenced object from memory.
    ///
    /// # Arguments
    ///
    ///  * `record_handle` - unique identifier of a record fetch request
    ///
    /// # Returns
    ///
    ///  * `ErrorCode`
    ///
    /// # ErrorCodes
    ///
    ///  * `Success` - Execution successful
    ///  * `InvalidRecordHandle` - Provided record handle does not exist
    ///
    pub fn free_record(&self, record_handle: i32) -> ErrorCode {
        if self.records.remove(record_handle) {
            ErrorCode::Success
        }
        else {
            ErrorCode::InvalidRecordHandle
        }
    }

    ///
    /// Removes a record handle, thus removing the referenced object from memory.
    ///
    /// # Arguments
    ///
    ///  * `record_handle` - unique identifier of a search result
    ///
    /// # Returns
    ///
    ///  * `ErrorCode`
    ///
    /// # ErrorCodes
    ///
    ///  * `Success` - Execution successful
    ///  * `InvalidSearchHandle` - Provided search handle does not exist
    ///
    pub fn free_search(&self, search_handle: i32) -> ErrorCode {
        if self.searches.remove(search_handle) {
            ErrorCode::Success
        }
        else {
            ErrorCode::InvalidSearchHandle
        }
    }

    ///
    /// Adds a new record identified by type and id.
    ///
    /// # Arguments
    ///
    ///  * `type_` - record type
    ///  * `id` - record id (name)
    ///  * `value` - record value
    ///  * `tags` - a map of (tag_name: tag_value) pairs
    ///
    /// # Returns
    ///
    ///  * `ErrorCode`
    ///
    /// # ErrorCodes
    ///
    ///  * `Success` - Execution successful
    ///  * `RecordAlreadExists` - Record with the provided type and id already exist in the DB
    ///  * `DatabaseError` - Unexpected error occurred while communicating with the DB
    ///
    pub fn add_record(&self, type_: &str, id: &str, value: &Vec<u8>, tags: &HashMap<String, String>) -> ErrorCode {
        // start an transaction
        let mut transaction = check_result!(
            self.write_pool.start_transaction(true, None, Some(false)),
            ErrorCode::DatabaseError);

        let item_id = {
            let result: Result<QueryResult, Error> = transaction.prep_exec(
                Statement::InsertRecord.as_str(),
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
                        Statement::InsertPlaintextTag.as_str(),
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
                        Statement::InsertEncryptedTag.as_str(),
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

    ///
    /// Fetches a record identified by type and id.
    ///
    /// # Arguments
    ///
    ///  * `type_` - record type
    ///  * `id` - record id (name)
    ///  * `options` - options in the form of {"fetch_value": true, "fetch_tags": true} determining whether to fetch the value and/or tags
    ///  * `record_handle_p` - user handle that will be used for accessing the fetched record
    ///
    /// # Returns
    ///
    ///  * `ErrorCode`
    ///
    /// # ErrorCodes
    ///
    ///  * `Success` - Execution successful
    ///  * `UnknownRecord` - Record with the provided type and id does not exist in the DB
    ///  * `DatabaseError` - Unexpected error occurred while communicating with the DB
    ///  * `InvalidEncoding` - Invalid encoding of a provided/fetched string
    ///
    pub fn fetch_record(&self, type_: &str, id: &str, options: &str, record_handle_p: *mut i32) -> ErrorCode {
        let options: FetchOptions = check_result!(serde_json::from_str(options), ErrorCode::InvalidJSON);

        let query = format!(
            "SELECT {}, {} \
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

        let row = check_result!(check_option!(result.next(), ErrorCode::UnknownRecord), ErrorCode::DatabaseError);

        // These 2 value cannot be NULL.
        let db_value: Vec<u8> = check_option!(row.get(0), ErrorCode::DatabaseError);
        let tags: String = check_option!(row.get(1), ErrorCode::DatabaseError);

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

    ///
    /// Returns a record with the specified handle.
    ///
    /// # Arguments
    ///
    ///  * `record_handle` - unique identifier of a record
    ///
    /// # Returns
    ///
    ///  * `Option<Arc<Record>>`
    ///
    pub fn get_record(&self, record_handle: i32) -> Option<Arc<Record>> {
        self.records.get(record_handle)
    }

    ///
    /// Deletes a record identified by type and id.
    ///
    /// # Arguments
    ///
    ///  * `type_` - record type
    ///  * `id` - record id (name)
    ///
    /// # Returns
    ///
    ///  * `ErrorCode`
    ///
    /// # ErrorCodes
    ///
    ///  * `Success` - Execution successful
    ///  * `UnknownRecord` - Record with the provided type and id does not exist in the DB
    ///  * `DatabaseError` - Unexpected error occurred while communicating with the DB
    ///
    pub fn delete_record(&self, type_: &str, id: &str) -> ErrorCode {
        let result: QueryResult = check_result!(
            self.write_pool.prep_exec(
                Statement::DeleteRecord.as_str(),
                params! {
                    "wallet_id" => self.wallet_id,
                    "type" => type_,
                    "name" => id
                }
            ),
            ErrorCode::DatabaseError
        );

        if result.affected_rows() != 1 {
            return ErrorCode::UnknownRecord;
        }

        ErrorCode::Success
    }

    ///
    /// Updates the value of a record identified by type and id.
    ///
    /// # Arguments
    ///
    ///  * `type_` - record type
    ///  * `id` - record id (name)
    ///  * `value` - new value
    ///
    /// # Returns
    ///
    ///  * `ErrorCode`
    ///
    /// # ErrorCodes
    ///
    ///  * `Success` - Execution successful
    ///  * `UnknownRecord` - Record with the provided type and id does not exist in the DB
    ///  * `DatabaseError` - Unexpected error occurred while communicating with the DB
    ///
    pub fn update_record_value(&self, type_: &str, id: &str, value: &Vec<u8>) -> ErrorCode {
        let result: QueryResult = check_result!(
            self.write_pool.prep_exec(
                Statement::UpdateRecordValue.as_str(),
                    params!{
                        "value" => value,
                        "type" => type_,
                        "name" => id
                    }
            ),
            ErrorCode::DatabaseError
        );

        // When performing updates MySQL return only the rows it has changed as affected_rows.
        // If the value provided for the UPDATE is the same as the one already in the DB MySQL will ignore that row.
        // Code below is checking if the user provided the same value as the one that is in the DB,
        // or if the record doesn't exist.
        if result.affected_rows() != 1 {
            let mut result: QueryResult = check_result!(
                self.read_pool.prep_exec(
                   Statement::CheckRecordExists.as_str(),
                     params!{
                        "type" => type_,
                        "name" => id
                     }
                ),
                ErrorCode::DatabaseError
            );

            let row = check_result!(check_option!(result.next(), ErrorCode::UnknownRecord), ErrorCode::DatabaseError);
            let found_rows: u64 = check_option!(row.get(0), ErrorCode::UnknownRecord);

            if found_rows != 1 {
                return ErrorCode::UnknownRecord;
            }
        }

        ErrorCode::Success
    }

    ///
    /// Adds tags for a record identified by type and id.
    ///
    /// # Arguments
    ///
    ///  * `type_` - record type
    ///  * `id` - record id (name)
    ///  * `tag_names` - a map containing (tag_name: tag_value) pairs
    ///
    /// # Returns
    ///
    ///  * `ErrorCode`
    ///
    /// # ErrorCodes
    ///
    ///  * `Success` - Execution successful
    ///  * `UnknownRecord` - Record with the provided type and id does not exist in the DB
    ///  * `TagAlreadyExists` - Provided tag already exists in the DB
    ///  * `TagDataTooLong` - Provided tag_name or tag_value exceed the size limit
    ///  * `DatabaseError` - Unexpected error occurred while communicating with the DB
    ///
    pub fn add_record_tags(&self, type_: &str, id: &str, tags: &HashMap<String, String>) -> ErrorCode {
        // Start a transaction
        let mut transaction = check_result!(
            self.write_pool.start_transaction(true, None, Some(false)),
            ErrorCode::DatabaseError
        );

        let item_id = {
            let mut result: QueryResult = check_result!(
                transaction.prep_exec(
                    Statement::GetRecordID.as_str(),
                    params!{
                        "type" => type_,
                        "name" => id
                    }
                ),
                ErrorCode::DatabaseError
            );

            let row = check_result!(check_option!(result.next(), ErrorCode::UnknownRecord), ErrorCode::DatabaseError);
            let item_id: u64 = check_option!(row.get(0), ErrorCode::UnknownRecord);

            item_id
        };

        for (tag_name, tag_value) in tags {
            let first_char = tag_name.chars().next().unwrap_or('\0');

            let result = match first_char {
                '~' => {
                    let mut tag_name = tag_name.clone();
                    tag_name.remove(0);

                    transaction.prep_exec(
                        Statement::InsertPlaintextTag.as_str(),
                        params!{
                            "name" => tag_name,
                            "value" => tag_value,
                            item_id
                        }
                    )
                },
                _ => {
                    transaction.prep_exec(
                        Statement::InsertEncryptedTag.as_str(),
                        params!{
                            "name" => tag_name,
                            "value" => tag_value,
                            item_id
                        }
                    )
                }
            };

            match result {
                Err(Error::MySqlError(err)) => {
                    match err.state.as_ref() {
                        "22001" => {return ErrorCode::TagDataTooLong},
                        "23000" => {return ErrorCode::TagAlreadyExists}
                        _ => {return ErrorCode::DatabaseError},
                    };
                },
                Err(_) => return ErrorCode::DatabaseError,
                Ok(result) => result,
            };
        }

        check_result!(transaction.commit(), ErrorCode::DatabaseError);

        ErrorCode::Success
    }

    ///
    /// Updates tags of a record identified by type and id.
    ///
    /// # Arguments
    ///
    ///  * `type_` - record type
    ///  * `id` - record id (name)
    ///  * `tag_names` - a map containing (tag_name: new_tag_value) pairs
    ///
    /// # Returns
    ///
    ///  * `ErrorCode`
    ///
    /// # ErrorCodes
    ///
    ///  * `Success` - Execution successful
    ///  * `UnknownRecord` - Record with the provided type and id does not exist in the DB
    ///  * `UnknownTag` - Tag name specified in the map does not exist in the DB
    ///  * `DatabaseError` - Unexpected error occurred while communicating with the DB
    ///
    pub fn update_record_tags(&self, type_: &str, id: &str, tags: &HashMap<String, String>) -> ErrorCode {
        // Start a transaction
        let mut transaction = check_result!(
            self.write_pool.start_transaction(true, None, Some(false)),
            ErrorCode::DatabaseError);

        let item_id = {
            let mut result: QueryResult = check_result!(
                transaction.prep_exec(Statement::GetRecordID.as_str(),
                    params!{
                        "type" => type_,
                        "name" => id
                    }
                ),
                ErrorCode::DatabaseError
            );

            let row = check_result!(check_option!(result.next(), ErrorCode::UnknownRecord), ErrorCode::DatabaseError);
            let item_id: u64 = check_option!(row.get(0), ErrorCode::UnknownRecord);

            item_id
        };

        for (tag_name, tag_value) in tags {
            let first_char = tag_name.chars().next().unwrap_or('\0');

            let affected_rows = {
                let result = match first_char {
                    '~' => {
                        let mut tag_name = tag_name.clone();
                        tag_name.remove(0);

                        transaction.prep_exec(
                            Statement::UpdatePlaintextTag.as_str(),
                            params!{
                                "value" => tag_value,
                                "name" => tag_name,
                                item_id
                            }
                        )

                    },
                    _ => {
                        transaction.prep_exec(
                            Statement::UpdateEncryptedTag.as_str(),
                            params!{
                                "value" => tag_value,
                                "name" => tag_name,
                                item_id
                            }
                        )
                    }
                };

                let result = match result {
                    Err(Error::MySqlError(err)) => {
                        match err.state.as_ref() {
                            "22001" => {return ErrorCode::TagDataTooLong},
                            _ => {return ErrorCode::DatabaseError},
                        };
                    },
                    Err(_) => return ErrorCode::DatabaseError,
                    Ok(result) => result,
                };

                result.affected_rows()

            };

            // When performing updates MySQL return only the rows it has changed as affected_rows.
            // If the value provided for the UPDATE is the same as the one already in the DB MySQL will ignore that row.
            // Code below is checking if the user provided the same value as the one that is in the DB,
            // or if the tag that is provided doesn't exist.
            if affected_rows != 1 {
                let mut result = match first_char {
                    '~' => {
                        let mut tag_name = tag_name.clone();
                        tag_name.remove(0);

                        check_result!(
                            transaction.prep_exec(Statement::CheckPlaintextTagExists.as_str(),
                                params!{
                                    "name" => tag_name,
                                    item_id
                                }
                            ),
                            ErrorCode::DatabaseError
                        )
                    },
                    _ => {
                        check_result!(
                            transaction.prep_exec(Statement::CheckEncryptedTagExists.as_str(),
                                params!{
                                    "name" => tag_name,
                                    item_id
                                }
                            ),
                            ErrorCode::DatabaseError
                        )
                    }
                };

                let row = check_result!(check_option!(result.next(), ErrorCode::UnknownTag), ErrorCode::DatabaseError);
                let found_rows: u64 = check_option!(row.get(0), ErrorCode::UnknownTag);

                if found_rows != 1 {
                    return ErrorCode::UnknownTag;
                }
            }
        }

        check_result!(transaction.commit(), ErrorCode::DatabaseError);

        ErrorCode::Success
    }

    ///
    /// Deletes tags of a record identified by type and id.
    ///
    /// # Arguments
    ///
    ///  * `type_` - record type
    ///  * `id` - record id (name)
    ///  * `tag_names` - list of tag names that need to be deleted
    ///
    /// # Returns
    ///
    ///  * `ErrorCode`
    ///
    /// # ErrorCodes
    ///
    ///  * `Success` - Execution successful
    ///  * `UnknownRecord` - Record with the provided type and id does not exist in the DB
    ///  * `UnknownTag` - Tag name specified in the list does not exist in the DB
    ///  * `DatabaseError` - Unexpected error occurred while communicating with the DB
    ///
    pub fn delete_record_tags(&self, type_: &str, id: &str, tag_names: &Vec<String>) -> ErrorCode {

        let mut transaction = check_result!(
            self.write_pool.start_transaction(true, None, Some(false)),
            ErrorCode::DatabaseError);

        let item_id = {
            let mut result: QueryResult = check_result!(
                transaction.prep_exec(Statement::GetRecordID.as_str(),
                    params!{
                        "type" => type_,
                        "name" => id
                    }
                ),
                ErrorCode::DatabaseError
            );

            let row = check_result!(check_option!(result.next(), ErrorCode::UnknownRecord), ErrorCode::DatabaseError);
            let item_id: u64 = check_option!(row.get(0), ErrorCode::UnknownRecord);

            item_id
        };

        for tag_name in tag_names {
            let first_char = tag_name.chars().next().unwrap_or('\0');

            let result = match first_char {
                '~' => {
                    let mut tag_name = tag_name.clone();
                    tag_name.remove(0);
                    check_result!(
                        transaction.prep_exec(Statement::DeletePlaintextTag.as_str(),
                            params!{
                                "name" => tag_name,
                                item_id
                            }
                        ),
                        ErrorCode::DatabaseError
                    )
                },
                _ => {
                    check_result!(
                        transaction.prep_exec(Statement::DeleteEncryptedTag.as_str(),
                            params!{
                                "name" => tag_name,
                                item_id
                            }
                        ),
                        ErrorCode::DatabaseError
                    )
                }
            };

            if result.affected_rows() != 1 {
                return ErrorCode::UnknownTag;
            }
        }

        check_result!(transaction.commit(), ErrorCode::DatabaseError);

        ErrorCode::Success
    }
}
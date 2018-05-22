pub mod statement;
mod query_translator;

use utils::handle_store::HandleStore;
use aurora_storage::statement::Statement;

use std::sync::{RwLock, Arc};
use mysql::{Pool, QueryResult, Error};
use errors::error_code::ErrorCode;
use std::collections::HashMap;
use std::ffi::CString;
use serde_json;

fn default_true() -> bool {
    true
}
fn default_false() -> bool {
    false
}

// TODO: Update documentation strings with new error codes

#[derive(Deserialize)]
pub struct FetchOptions {
    #[serde(default="default_false")]
    fetch_type: bool,

    #[serde(default="default_true")]
    fetch_value: bool,

    #[serde(default="default_true")]
    fetch_tags: bool,
}

pub struct Search<'a> {
    pub search_result: RwLock<QueryResult<'a>>,
}

impl<'a> Search<'a> {
    fn new(search_result: QueryResult<'a>) -> Self {
        Self{ search_result: RwLock::new(search_result) }
    }
}

#[derive(Debug)]
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

pub struct AuroraStorage<'a> {
    wallet_id: u64,
    records: HandleStore<Record>,
    searches: HandleStore<Search<'a>>,
    metadata: HandleStore<CString>,
    read_pool: Arc<Pool>, // cached reference to the pool
    write_pool: Arc<Pool>,
}

impl<'a> AuroraStorage<'a> {
    pub fn new(wallet_id: u64, read_pool: Arc<Pool>, write_pool: Arc<Pool>) -> Self {
        Self{wallet_id, records: HandleStore::new(), searches: HandleStore::new(), metadata: HandleStore::new(), read_pool, write_pool}
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
            ErrorCode::InvalidState
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
            ErrorCode::InvalidState
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
    ///  * `IOError` - Unexpected error occurred while communicating with the DB
    ///
    pub fn add_record(&self, type_: &str, id: &str, value: &Vec<u8>, tags: &HashMap<String, serde_json::Value>) -> ErrorCode {
        // start an transaction
        let mut transaction = check_result!(
            self.write_pool.start_transaction(true, None, Some(false)),
            ErrorCode::IOError);

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
                Err(Error::MySqlError(_)) => return ErrorCode::RecordAlreadyExists,
                Err(_) => return ErrorCode::IOError,
                Ok(result) => result,
            };

            result.last_insert_id()
        };

        for (tag_name, tag_value) in tags {
            let first_char = tag_name.chars().next().unwrap_or('\0');

            let result = if first_char == '~' { // plain text
                let mut tag_name = tag_name.clone();
                tag_name.remove(0);

                match tag_value {
                    &serde_json::Value::String(ref tag_value) => {
                        transaction.prep_exec(
                            Statement::UpsertPlaintextTag.as_str(),
                            params!{
                                "name" => tag_name,
                                "value" => tag_value,
                                "item_id" => item_id
                            }
                        )
                    },
                    _ => {
                        // TODO: Non String Tag Handling
                        return ErrorCode::InvalidStructure;
                    }
                }
            }
            else {
                transaction.prep_exec(
                    Statement::UpsertEncryptedTag.as_str(),
                    params!{
                        "name" => tag_name,
                        "value" => tag_value.as_str(),
                        "item_id" => item_id
                    }
                )
            };

            match result {
                Err(Error::MySqlError(err)) => {
                    match err.state.as_ref() {
                        "22001" => {return ErrorCode::InvalidStructure },
                        _ => {return ErrorCode::IOError },
                    };
                },
                Err(_) => return ErrorCode::IOError,
                Ok(result) => result,
            };
        }

        check_result!(transaction.commit(), ErrorCode::IOError);

        ErrorCode::Success
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
    ///  * `IOError` - Unexpected error occurred while communicating with the DB
    ///
    pub fn add_record_1(&self, type_: &str, id: &str, value: &Vec<u8>, tags: &str) -> ErrorCode {

        let result = {
            self.write_pool.prep_exec(
                        Statement::InsertRecord_1.as_str(),
                        params!{
                            "type" => type_,
                            "name" => id,
                            "value" => value,
                            "tags" => tags,
                            "wallet_id" => self.wallet_id
                        }
                )
        };

        let result: QueryResult = match result {
            Err(Error::MySqlError(err)) => { println!("{:?}", err); return ErrorCode::RecordAlreadyExists; },
            Err(_) => return ErrorCode::IOError,
            Ok(result) => result,
        };

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
    ///  * `record_handle_p` - output param - handle that will be used for accessing the fetched record
    ///
    /// # Returns
    ///
    ///  * `ErrorCode`
    ///
    /// # ErrorCodes
    ///
    ///  * `Success` - Execution successful
    ///  * `UnknownRecord` - Record with the provided type and id does not exist in the DB
    ///  * `IOError` - Unexpected error occurred while communicating with the DB
    ///  * `InvalidEncoding` - Invalid encoding of a provided/fetched string
    ///
    pub fn fetch_record(&self, type_: &str, id: &str, options: &str, record_handle_p: *mut i32) -> ErrorCode {
        let options: FetchOptions = check_result!(serde_json::from_str(options), ErrorCode::InvalidStructure);

        let query = format!(
            "SELECT {}, {} \
                FROM items i \
                WHERE wallet_id = :wallet_id \
                    AND type = :type \
                    AND name = :name",
            if options.fetch_value { "value" }
                else {"''"},
            if options.fetch_tags {
                "CONCAT(\
                    '{', \
                    CONCAT_WS(\
                        ',', \
                        (select group_concat(concat(json_quote(name), ':', json_quote(value))) from tags_encrypted WHERE item_id = i.id), \
                        (select group_concat(concat(json_quote(concat('~', name)), ':', json_quote(value))) from tags_plaintext WHERE item_id = i.id)\
                    ), \
                '}') tags"
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
            ErrorCode::IOError
        );

        let row = check_result!(check_option!(result.next(), ErrorCode::WalletNotFoundError), ErrorCode::IOError);

        // These 2 value cannot be NULL.
        let db_value: Vec<u8> = check_option!(row.get(0), ErrorCode::IOError);
        let tags: String = check_option!(row.get(1), ErrorCode::IOError);

        let record = Record::new(
            check_result!(CString::new(id), ErrorCode::InvalidState),
            if options.fetch_value {Some(db_value)} else {None},
            if options.fetch_tags {Some(check_result!(CString::new(tags), ErrorCode::InvalidState))} else {None},
            None
        );

        let record_handle = self.records.insert(record);

        unsafe { *record_handle_p = record_handle; }

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
    ///  * `record_handle_p` - output param - handle that will be used for accessing the fetched record
    ///
    /// # Returns
    ///
    ///  * `ErrorCode`
    ///
    /// # ErrorCodes
    ///
    ///  * `Success` - Execution successful
    ///  * `UnknownRecord` - Record with the provided type and id does not exist in the DB
    ///  * `IOError` - Unexpected error occurred while communicating with the DB
    ///  * `InvalidEncoding` - Invalid encoding of a provided/fetched string
    ///
    pub fn fetch_record_1(&self, type_: &str, id: &str, options: &str, record_handle_p: *mut i32) -> ErrorCode {
        let options: FetchOptions = check_result!(serde_json::from_str(options), ErrorCode::InvalidStructure);

        let query = format!(
            "SELECT {}, {} FROM items_1 i \
                WHERE \
                    wallet_id = :wallet_id \
                    AND type = :type \
                    AND name = :name",
            if options.fetch_value { "value" } else {"''"},
            if options.fetch_tags { "tags" } else {"''"}
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
            ErrorCode::IOError
        );

        let row = check_result!(check_option!(result.next(), ErrorCode::WalletNotFoundError), ErrorCode::IOError);

        // These 2 value cannot be NULL.
        let db_value: Vec<u8> = check_option!(row.get(0), ErrorCode::IOError);
        let tags: String = check_option!(row.get(1), ErrorCode::IOError);

        let record = Record::new(
            check_result!(CString::new(id), ErrorCode::InvalidState),
            if options.fetch_value {Some(db_value)} else {None},
            if options.fetch_tags {Some(check_result!(CString::new(tags), ErrorCode::InvalidState))} else {None},
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
    ///  * `IOError` - Unexpected error occurred while communicating with the DB
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
            ErrorCode::IOError
        );

        if result.affected_rows() != 1 {
            return ErrorCode::WalletNotFoundError;
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
    ///  * `IOError` - Unexpected error occurred while communicating with the DB
    ///
    pub fn update_record_value(&self, type_: &str, id: &str, value: &Vec<u8>) -> ErrorCode {
        let result: QueryResult = check_result!(
            self.write_pool.prep_exec(
                Statement::UpdateRecordValue.as_str(),
                    params!{
                        "value" => value,
                        "type" => type_,
                        "name" => id,
                        "wallet_id" => self.wallet_id
                    }
            ),
            ErrorCode::IOError
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
                        "name" => id,
                        "wallet_id" => self.wallet_id
                     }
                ),
                ErrorCode::IOError
            );

            let row = check_result!(check_option!(result.next(), ErrorCode::WalletNotFoundError), ErrorCode::IOError);
            let found_rows: u64 = check_option!(row.get(0), ErrorCode::WalletNotFoundError);

            if found_rows != 1 {
                return ErrorCode::WalletNotFoundError;
            }
        }

        ErrorCode::Success
    }

    ///
    /// Adds tags for a record identified by type and id.
    /// If tag with that name exists it will be updated.
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
    ///  * `IOError` - Unexpected error occurred while communicating with the DB
    ///
    pub fn add_record_tags(&self, type_: &str, id: &str, tags: &HashMap<String, serde_json::Value>) -> ErrorCode {
        // Start a transaction
        let mut transaction = check_result!(
            self.write_pool.start_transaction(true, None, Some(false)),
            ErrorCode::IOError
        );

        let item_id = {
            let mut result: QueryResult = check_result!(
                transaction.prep_exec(
                    Statement::GetRecordID.as_str(),
                    params!{
                        "type" => type_,
                        "name" => id,
                        "wallet_id" => self.wallet_id
                    }
                ),
                ErrorCode::IOError
            );

            let row = check_result!(check_option!(result.next(), ErrorCode::WalletNotFoundError), ErrorCode::IOError);
            let item_id: u64 = check_option!(row.get(0), ErrorCode::WalletNotFoundError);

            item_id
        };

        for (tag_name, tag_value) in tags {
            let first_char = tag_name.chars().next().unwrap_or('\0');

            let result = match first_char {
                '~' => {
                    let mut tag_name = tag_name.clone();
                    tag_name.remove(0);

                    match tag_value {
                        &serde_json::Value::String(ref tag_value) => {
                            transaction.prep_exec(
                                Statement::UpsertPlaintextTag.as_str(),
                                params!{
                                    "name" => tag_name,
                                    "value" => tag_value,
                                    item_id
                                }
                            )
                        },
                        _ => {
                            // TODO: Non String Tag Handling
                            return ErrorCode::InvalidStructure;
                        }
                    }
                },
                _ => {
                    transaction.prep_exec(
                        Statement::UpsertEncryptedTag.as_str(),
                        params!{
                            "name" => tag_name,
                            "value" => tag_value.as_str(),
                            item_id
                        }
                    )
                }
            };

            match result {
                Err(Error::MySqlError(err)) => {
                    match err.state.as_ref() {
                        "22001" => {return ErrorCode::InvalidStructure },
                        _ => {return ErrorCode::IOError },
                    };
                },
                Err(_) => return ErrorCode::IOError,
                Ok(result) => result,
            };
        }

        check_result!(transaction.commit(), ErrorCode::IOError);

        ErrorCode::Success
    }

    ///
    /// Updates tags of a record identified by type and id.
    /// This function will replace all tags with new.
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
    ///  * `IOError` - Unexpected error occurred while communicating with the DB
    ///
    pub fn update_record_tags(&self, type_: &str, id: &str, tags: &HashMap<String, serde_json::Value>) -> ErrorCode {
        // Start a transaction
        let mut transaction = check_result!(
            self.write_pool.start_transaction(true, None, Some(false)),
            ErrorCode::IOError);

        let item_id = {
            let mut result: QueryResult = check_result!(
                transaction.prep_exec(Statement::GetRecordID.as_str(),
                    params!{
                        "type" => type_,
                        "name" => id,
                        "wallet_id" => self.wallet_id
                    }
                ),
                ErrorCode::IOError
            );

            let row = check_result!(check_option!(result.next(), ErrorCode::WalletNotFoundError), ErrorCode::IOError);
            let item_id: u64 = check_option!(row.get(0), ErrorCode::WalletNotFoundError);

            item_id
        };

        // Delete all existing tags.
        {
            check_result!(transaction.prep_exec(Statement::DeleteAllPlaintextTags.as_str(), params!{item_id}), ErrorCode::IOError);
            check_result!(transaction.prep_exec(Statement::DeleteAllEncryptedTags.as_str(), params!{item_id}), ErrorCode::IOError);
        }

        for (tag_name, tag_value) in tags {
            let first_char = tag_name.chars().next().unwrap_or('\0');

            let result = match first_char {
                '~' => {
                    let mut tag_name = tag_name.clone();
                    tag_name.remove(0);

                    match tag_value {
                        &serde_json::Value::String(ref tag_value) => {
                            transaction.prep_exec(
                                Statement::UpsertPlaintextTag.as_str(),
                                params!{
                                    "value" => tag_value,
                                    "name" => tag_name,
                                    item_id
                                }
                            )
                        },
                        _ => {
                            // TODO Non String Tag Handling
                            return ErrorCode::InvalidStructure;
                        }
                    }
                },
                _ => {
                    transaction.prep_exec(
                        Statement::UpsertEncryptedTag.as_str(),
                        params!{
                            "value" => tag_value.as_str(),
                            "name" => tag_name,
                            item_id
                        }
                    )
                }
            };

             match result {
                Err(Error::MySqlError(err)) => {
                    match err.state.as_ref() {
                        "22001" => {return ErrorCode::InvalidStructure },
                        _ => {return ErrorCode::IOError },
                    };
                },
                Err(_) => return ErrorCode::IOError,
                Ok(result) => result,
            };
        }

        check_result!(transaction.commit(), ErrorCode::IOError);

        ErrorCode::Success
    }

    ///
    /// Updates tags of a record identified by type and id.
    /// This function will replace all tags with new.
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
    ///  * `IOError` - Unexpected error occurred while communicating with the DB
    ///
    pub fn update_record_tags_1(&self, type_: &str, id: &str, tags: &str) -> ErrorCode {

        let result = {
            self.write_pool.prep_exec(
                        Statement::UpdateTags.as_str(),
                        params!{
                            "tags" => tags,
                            "type" => type_,
                            "name" => id,
                            "wallet_id" => self.wallet_id
                        }
                )
        };

        let result: QueryResult = match result {
            Err(Error::MySqlError(err)) => { println!("{:?}", err); return ErrorCode::InvalidStructure; },
            Err(_) => return ErrorCode::IOError,
            Ok(result) => result,
        };

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
    ///  * `IOError` - Unexpected error occurred while communicating with the DB
    ///
    pub fn delete_record_tags(&self, type_: &str, id: &str, tag_names: &Vec<String>) -> ErrorCode {

        let mut transaction = check_result!(
            self.write_pool.start_transaction(true, None, Some(false)),
            ErrorCode::IOError);

        let item_id = {
            let mut result: QueryResult = check_result!(
                transaction.prep_exec(Statement::GetRecordID.as_str(),
                    params!{
                        "type" => type_,
                        "name" => id,
                        "wallet_id" => self.wallet_id
                    }
                ),
                ErrorCode::IOError
            );

            let row = check_result!(check_option!(result.next(), ErrorCode::WalletNotFoundError), ErrorCode::IOError);
            let item_id: u64 = check_option!(row.get(0), ErrorCode::WalletNotFoundError);

            item_id
        };

        for tag_name in tag_names {
            let first_char = tag_name.chars().next().unwrap_or('\0');

            match first_char {
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
                        ErrorCode::IOError
                    );
                },
                _ => {
                    check_result!(
                        transaction.prep_exec(Statement::DeleteEncryptedTag.as_str(),
                            params!{
                                "name" => tag_name,
                                item_id
                            }
                        ),
                        ErrorCode::IOError
                    );
                }
            }
        }

        check_result!(transaction.commit(), ErrorCode::IOError);

        ErrorCode::Success
    }

    pub fn get_metadata(&self) -> Result<(Arc<CString>, i32), ErrorCode> {
        let mut result: QueryResult = check_result!(
            self.read_pool.prep_exec(
                Statement::GetMetadata.as_str(),
                params! {
                    "wallet_id" => self.wallet_id,
                }
            ),
            Err(ErrorCode::IOError)
        );

        let row = check_result!(check_option!(result.next(), Err(ErrorCode::WalletNotFoundError)), Err(ErrorCode::IOError));
        let metadata: String = check_option!(row.get(0), Err(ErrorCode::IOError));
        let metadata = check_result!(CString::new(metadata), Err(ErrorCode::InvalidState));

        let handle = self.metadata.insert(metadata);
        let metadata = check_option!(self.metadata.get(handle), Err(ErrorCode::WalletNotFoundError));

        Ok((metadata, handle))
    }

    pub fn set_metadata(&self, metadata: &str) -> ErrorCode {
        check_result!(
            self.write_pool.prep_exec(
                Statement::SetMetadata.as_str(),
                params! {
                    "wallet_id" => self.wallet_id,
                    "metadata" => metadata,
                }
            ),
            ErrorCode::IOError
        );

        ErrorCode::Success
    }

    pub fn free_metadata(&self, metadata_handle: i32) -> ErrorCode {
        if self.metadata.remove(metadata_handle) {
            ErrorCode::Success
        }
        else {
            ErrorCode::InvalidState
        }
    }

    ///
    /// Performs a search defined by the user query and options.
    ///
    /// # Arguments
    ///
    ///  * `type_` - type of the record that we are searching for
    ///  * `query_json` - query conditions specified in the form of a json
    ///         {
    ///             "tagName": "tagValue",
    ///             $or: {
    ///                 "tagName2": { $regex: 'pattern' },
    ///                 "tagName3": { $gte: 123 },
    ///             },
    ///         }
    ///  * `options_json` - options specifying what attributes ought to be fetched ex.
    ///         {
    ///             fetch_type: (optional, true by default)
    ///             fetch_value: (optional, true by default)
    ///             fetch_value: (optional, true by default)
    ///         }
    ///  * `search_handle_p` - output param - handle that will be used for accessing the search result
    ///
    /// # Returns
    ///
    ///  * `ErrorCode`
    ///
    /// # ErrorCodes
    ///
    ///  * `Success` - Execution successful
    ///  * `IOError` - Unexpected error occurred while communicating with the DB
    ///  * `InvalidStructure` - Invalid structure of the JSON arguments -> query | options
    ///
    pub fn search_records(&self, type_: &str, query_json: &str, options_json: &str, search_handle_p: *mut i32) -> ErrorCode {
        let fetch_options: FetchOptions = check_result!(serde_json::from_str(options_json), ErrorCode::InvalidStructure);

        let wql = check_option!(query_translator::parse_from_json(&query_json), ErrorCode::InvalidStructure);
        let (query, arguments) = check_option!(query_translator::wql_to_sql(self.wallet_id, type_, &wql, &fetch_options), ErrorCode::InvalidStructure);

        let search_result: QueryResult = check_result!(
            self.read_pool.prep_exec(query, arguments),
            ErrorCode::IOError
        );

        let search_handle = self.searches.insert(Search::new(search_result));

        unsafe { *search_handle_p = search_handle; }

        ErrorCode::Success
    }

    ///
    /// Performs a search that grabs all records of a wallet with all attributes from the DB.
    ///
    /// # Arguments
    ///
    ///  * `search_handle_p` - output param - handle that will be used for accessing the search result
    ///
    /// # Returns
    ///
    ///  * `ErrorCode`
    ///
    /// # ErrorCodes
    ///
    ///  * `Success` - Execution successful
    ///  * `IOError` - Unexpected error occurred while communicating with the DB
    ///
    pub fn search_all_records(&self, search_handle_p: *mut i32) -> ErrorCode {
         let search_result: QueryResult = check_result!(
            self.read_pool.prep_exec(
                Statement::GetAllRecords.as_str(),
                params! {
                    "wallet_id" => self.wallet_id,
                }
            ),
            ErrorCode::IOError
        );

        let search_handle = self.searches.insert(Search::new(search_result));

        unsafe { *search_handle_p = search_handle; }

        ErrorCode::Success
    }

    ///
    /// Fetches a new record from the search result set.
    ///
    /// # Arguments
    ///
    ///  * `search_handle` - unique identifier of a search request
    ///  * `record_handle_p` - output param - handle that will be used for accessing the record
    ///
    /// # Returns
    ///
    ///  * `ErrorCode`
    ///
    /// # ErrorCodes
    ///
    ///  * `Success` - Execution successful
    ///  * `InvalidState` - Provided search handle does not exist, or parsing of data has gone wrong
    ///  * `IOError` - Unexpected error occurred while communicating with the DB
    ///  * `WalletItemNotFound` - Result set exhausted, no more records to fetch
    ///
    pub fn fetch_search_next_record(&self, search_handle: i32, record_handle_p: *mut i32) -> ErrorCode {
        let search = check_option!(self.searches.get(search_handle), ErrorCode::InvalidState);

        let mut search_result = check_result!(search.search_result.write(), ErrorCode::IOError);

        let next_result = check_option!(search_result.next(), ErrorCode::WalletItemNotFound);

        let row = check_result!(next_result, ErrorCode::IOError);

        let record_type: Option<String> = check_option!(row.get(0), ErrorCode::IOError);
        let record_id: String = check_option!(row.get(1), ErrorCode::IOError);
        let record_value: Option<Vec<u8>> = check_option!(row.get(2), ErrorCode::IOError);
        let record_tags: Option<String> = check_option!(row.get(3), ErrorCode::IOError);

        let record = Record::new(
            check_result!(CString::new(record_id), ErrorCode::InvalidState),
            record_value,
            if let Some(record_tags) = record_tags { Some(check_result!(CString::new(record_tags), ErrorCode::InvalidState)) } else { None },
            if let Some(record_type) = record_type { Some(check_result!(CString::new(record_type), ErrorCode::InvalidState)) } else { None },
        );

        let record_handle = self.records.insert(record);

        unsafe { *record_handle_p = record_handle; }

        ErrorCode::Success
    }
}

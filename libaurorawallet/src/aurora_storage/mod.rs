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

#[derive(Deserialize, Debug)]
pub struct FetchOptions {
    #[serde(default="default_false", rename="retrieveType")]
    retrieve_type: bool,

    #[serde(default="default_true", rename="retrieveValue")]
    retrieve_value: bool,

    #[serde(default="default_true", rename="retrieveTags")]
    retrieve_tags: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchOptions {
    #[serde(default="default_true", rename="retrieveRecords")]
    pub retrieve_records: bool,

    #[serde(default="default_false", rename="retrieveTotalCount")]
    pub retrieve_total_count: bool,

    #[serde(default="default_false", rename="retrieveType")]
    pub retrieve_type: bool,

    #[serde(default="default_true", rename="retrieveValue")]
    pub retrieve_value: bool,

    #[serde(default="default_false", rename="retrieveTags")]
    pub retrieve_tags: bool,
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
    pub fn add_record(&self, type_: &str, id: &str, value: &Vec<u8>, tags: &str) -> ErrorCode {

        let result = {
            self.write_pool.prep_exec(
                        Statement::InsertRecord.as_str(),
                        params!{
                            "type" => type_,
                            "name" => id,
                            "value" => value,
                            "tags" => tags,
                            "wallet_id" => self.wallet_id
                        }
                )
        };

        match result {
                Err(Error::MySqlError(err)) => {
                    match err.code {
                        1062 => return ErrorCode::RecordAlreadyExists,
                        1406 => return ErrorCode::InvalidStructure,
                        _ => return ErrorCode::IOError,
                    };
                },
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

        let record: Record;

        if options.retrieve_value | options.retrieve_tags {
            let query = format!(
                "SELECT {}, {} FROM items i \
                    WHERE \
                        wallet_id = :wallet_id \
                        AND type = :type \
                        AND name = :name",
                if options.retrieve_value { "value" } else {"''"},
                if options.retrieve_tags { "tags" } else {"''"}
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

            // These 2 values cannot be NULL.
            let db_value: Vec<u8> = check_option!(row.get(0), ErrorCode::IOError);
            let tags: String = check_option!(row.get(1), ErrorCode::IOError);

            record = Record::new(
                check_result!(CString::new(id), ErrorCode::InvalidState),
                if options.retrieve_value {Some(db_value)} else {None},
                if options.retrieve_tags {Some(check_result!(CString::new(tags), ErrorCode::InvalidState))} else {None},
                Some(check_result!(CString::new(type_), ErrorCode::InvalidState))
            );
        } else {
            record = Record::new(
                check_result!(CString::new(id), ErrorCode::InvalidState),
                None,
                None,
                Some(check_result!(CString::new(type_), ErrorCode::InvalidState))
            );
        }

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
                    "type" => type_,
                    "name" => id,
                    "wallet_id" => self.wallet_id,
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

        let mut tag_name_value_paths: Vec<String> = Vec::new();

        for (tag_name, tag_value) in tags {
            let tag_name_path = format!(r#"'$."{}"', {}"#, tag_name, tag_value);
            tag_name_value_paths.push(tag_name_path);
        }

        let tag_name_value_paths = tag_name_value_paths.join(",");

        let query = format!("UPDATE items \
                            SET tags = JSON_SET(tags, {}) \
                            WHERE type = :type \
                            AND name = :name \
                            AND wallet_id = :wallet_id",
                            tag_name_value_paths
        );

        let result = {
            self.write_pool.prep_exec(
                        query,
                        params!{
                            "type" => type_,
                            "name" => id,
                            "wallet_id" => self.wallet_id
                        }
                )
        };

        let result = match result {
            Err(Error::MySqlError(_)) => return ErrorCode::InvalidStructure,
            Err(_) => return ErrorCode::IOError,
            Ok(result) => result,
        };

        if result.affected_rows() != 1 {
            return ErrorCode::WalletNotFoundError;
        }

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
    pub fn update_record_tags(&self, type_: &str, id: &str, tags: &str) -> ErrorCode {

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

        let result = match result {
            Err(Error::MySqlError(_)) => return ErrorCode::InvalidStructure,
            Err(_) => return ErrorCode::IOError,
            Ok(result) => result,
        };

        if result.affected_rows() != 1 {
            return ErrorCode::WalletNotFoundError;
        }

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

        let mut tag_name_paths: Vec<String> = Vec::new();

        for tag_name in tag_names {
            let tag_name_path = format!(r#"'$."{}"'"#, tag_name);
            tag_name_paths.push(tag_name_path);
        }

        let tag_name_paths = tag_name_paths.join(",");

        let query = format!("UPDATE items \
                            SET tags = JSON_REMOVE(tags, {}) \
                            WHERE type = :type \
                            AND name = :name \
                            AND wallet_id = :wallet_id",
                            tag_name_paths
        );


        let result = {
            self.write_pool.prep_exec(
                        query,
                        params!{
                            "type" => type_,
                            "name" => id,
                            "wallet_id" => self.wallet_id
                        }
                )
        };

        let result = match result {
            Err(Error::MySqlError(_)) => return ErrorCode::InvalidStructure,
            Err(_) => return ErrorCode::IOError,
            Ok(result) => result,
        };

        if result.affected_rows() != 1 {
            return ErrorCode::WalletNotFoundError;
        }

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
    ///             retireveType: (optional, true by default)
    ///             retrieveValue: (optional, true by default)
    ///             retireveTags: (optional, true by default)
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
        let search_options: SearchOptions = check_result!(serde_json::from_str(options_json), ErrorCode::InvalidStructure);

        // TODO: implement options.retrieve_total_count

        if !search_options.retrieve_records { // && !search_options.retrieve_total_count {
            return ErrorCode::InvalidStructure;
        }

        let wql = check_result!(query_translator::parse_from_json(&query_json), ErrorCode::InvalidStructure);
        let (query, arguments) = check_result!(query_translator::wql_to_sql(self.wallet_id, type_, &wql, &search_options), ErrorCode::InvalidStructure);

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
    ///  * `WalletNotFoundError` - Result set exhausted, no more records to fetch
    ///
    pub fn fetch_search_next_record(&self, search_handle: i32, record_handle_p: *mut i32) -> ErrorCode {
        let search = check_option!(self.searches.get(search_handle), ErrorCode::InvalidState);

        let mut search_result = check_result!(search.search_result.write(), ErrorCode::IOError);

        let next_result = check_option!(search_result.next(), ErrorCode::WalletNotFoundError);

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

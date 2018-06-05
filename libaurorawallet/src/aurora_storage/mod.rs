mod query_translator;
use utils::handle_store::HandleStore;
use utils::multi_pool::{MultiPool, StorageCredentials, StorageConfig};

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

#[derive(Deserialize)]
pub struct FetchOptions {
    #[allow(dead_code)]
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

#[derive(Debug)]
pub struct Search<'a> {
    pub search_result: Option<RwLock<QueryResult<'a>>>,
    pub total_count: Option<usize>,
}

impl<'a> Search<'a> {
    fn new(search_result: Option<QueryResult<'a>>, total_count: Option<usize>) -> Self {
        Self {
            search_result: search_result.map(|result|{RwLock::new(result)}),
            total_count: total_count
        }
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

lazy_static! {
    static ref CONNECTIONS: MultiPool = MultiPool::new();
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
    /// Creates a wallet with the given name in the DB specified in the config.
    ///
    /// # Arguments
    ///
    ///  * `name` - name of the wallet to be created
    ///  * `config` - json containing information like db_host, db_port, db_name
    ///  * `credentials` - json containing information about user and password for db access
    ///  * `metadata` - additional data that wil be stored along with the wallet
    ///
    /// # Returns
    ///
    ///  * `ErrorCode`
    ///
    /// # ErrorCodes
    ///
    ///  * `Success` - Execution successful
    ///  * `InvalidStructure` -  Invalid structure of the JSON arguments -> config | credentials
    ///  * `WalletAlreadyExistsError` - Wallet with the provided name already exists in the DB
    ///  * `IOError` - Unexpected error occurred while communicating with the DB
    ///
    pub fn create_storage(name: &str, config: &str, credentials: &str, metadata: &str) -> ErrorCode {

        trace!("Creating Storage -> name: {}, metadata: {}", name, metadata);

        let config: StorageConfig = check_result!(serde_json::from_str(config), ErrorCode::InvalidStructure);
        let credentials: StorageCredentials = check_result!(serde_json::from_str(credentials), ErrorCode::InvalidStructure);

        let write_pool = check_option!(CONNECTIONS.get(false, &config, &credentials), ErrorCode::IOError);

        let result = write_pool.prep_exec(
                        "INSERT INTO wallets(name, metadata) VALUES (:name, :metadata)",
                         params!{
                            name,
                            metadata
                         }
         );

        match result {
                Err(Error::MySqlError(err)) => {
                    warn!("MySQL Error while executing query. Err Code: {}, Err State: {}", err.code, err.state);
                    match err.code {
                        1062 => return ErrorCode::WalletAlreadyExistsError,
                        _ => return ErrorCode::IOError,
                    };
                },
                Err(err) => {
                    warn!("Unexpected Error while executing query. Err: {:?}", err);
                    return ErrorCode::IOError
                },
                Ok(result) => result,
        };

        trace!("Success Creating Storage with the name: {}", name);

        ErrorCode::Success
    }

    ///
    /// Opens a wallet with the given name in the DB specified in the config.
    ///
    /// # Arguments
    ///
    ///  * `name` - name of the wallet to be created
    ///  * `config` - json containing information like db_host, db_port, db_name
    ///  * `credentials` - json containing information about user and password for db access
    ///
    /// # Returns
    ///
    ///  * `ErrorCode`
    ///
    /// # ErrorCodes
    ///
    ///  * `Success` - Execution successful
    ///  * `InvalidStructure` -  Invalid structure of the JSON arguments -> config | credentials
    ///  * `InvalidState` - No wallet with the given name found in the DB
    ///  * `IOError` - Unexpected error occurred while communicating with the DB
    ///
    pub fn open_storage(name: &str, config: &str, credentials: &str) -> Result<Self, ErrorCode> {

        trace!("Opening Storage -> name: {}", name);

        let config: StorageConfig = check_result!(serde_json::from_str(config), Err(ErrorCode::InvalidStructure));
        let credentials: StorageCredentials = check_result!(serde_json::from_str(credentials), Err(ErrorCode::InvalidStructure));

        let read_pool = check_option!(CONNECTIONS.get(true, &config, &credentials), Err(ErrorCode::IOError));
        let write_pool = check_option!(CONNECTIONS.get(false, &config, &credentials), Err(ErrorCode::IOError));

        let mut result = check_result!(
                            read_pool.prep_exec(
                                "SELECT id FROM wallets WHERE name = :name",
                                params!{
                                    name
                                 }
                            ), Err(ErrorCode::IOError)
        );

        let wallet_id: u64 = check_option!(check_result!(check_option!(result.next(), Err(ErrorCode::InvalidState)), Err(ErrorCode::IOError)).get(0), Err(ErrorCode::InvalidState));

        trace!("Success Opening Storage with the name: {}", name);

        Ok(AuroraStorage::new(wallet_id, read_pool, write_pool))
    }

    ///
    /// Deletes a wallet with the given name in the DB specified in the config.
    ///
    /// # Arguments
    ///
    ///  * `name` - name of the wallet to be created
    ///  * `config` - json containing information like db_host, db_port, db_name
    ///  * `credentials` - json containing information about user and password for db access
    ///
    /// # Returns
    ///
    ///  * `ErrorCode`
    ///
    /// # ErrorCodes
    ///
    ///  * `Success` - Execution successful
    ///  * `InvalidStructure` -  Invalid structure of the JSON arguments -> config | credentials
    ///  * `InvalidState` - Wallet with the provided name does not exist in the DB
    ///  * `IOError` - Unexpected error occurred while communicating with the DB
    ///
    pub fn delete_storage(name: &str, config: &str, credentials: &str) -> ErrorCode {

        trace!("Deleting Storage -> name: {}", name);

        let config: StorageConfig = check_result!(serde_json::from_str(config), ErrorCode::InvalidStructure);
        let credentials: StorageCredentials = check_result!(serde_json::from_str(credentials), ErrorCode::InvalidStructure);

        let write_pool = check_option!(CONNECTIONS.get(false, &config, &credentials), ErrorCode::IOError);

        let result = check_result!(
                        write_pool.prep_exec(
                            "DELETE FROM wallets WHERE name = :name",
                             params!{
                                name
                             }
                        ), ErrorCode::IOError
        );

        if result.affected_rows() != 1 {
            warn!("Trying to delete a non existent storage, name: {}", name);
            return ErrorCode::InvalidState;
        }

        trace!("Success Deleting Storage with the name: {}", name);

        ErrorCode::Success
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
    ///  * `InvalidState` - Provided record handle does not exist
    ///
    pub fn free_record(&self, record_handle: i32) -> ErrorCode {

        trace!("Freeing Record -> record_handle: {}", record_handle);

        if self.records.remove(record_handle) {
            trace!("Success Freeing Record with the record_handle: {}", record_handle);
            ErrorCode::Success
        }
        else {
            warn!("Trying to free a non existent Record, record_handle: {}", record_handle);
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
    ///  * `InvalidState` - Provided search handle does not exist
    ///
    pub fn free_search(&self, search_handle: i32) -> ErrorCode {

        trace!("Freeing Search Result -> search_handle: {}", search_handle);

        if self.searches.remove(search_handle) {
            trace!("Success Freeing Search Result with the search_handle: {}", search_handle);
            ErrorCode::Success
        }
        else {
            warn!("Trying to free a non existent Search Result, search_handle: {}", search_handle);
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
    ///  * `ItemAlreadyExists` - Record with the provided type and id already exist in the DB
    ///  * `InvalidStructure` - Invalid structure of the JSON arguments -> tags
    ///  * `IOError` - Unexpected error occurred while communicating with the DB
    ///
    pub fn add_record(&self, type_: &str, id: &str, value: &Vec<u8>, tags: &str) -> ErrorCode {

        trace!("Adding Record -> type: {}, id: {}, value: {:?}, tags: {}", type_, id, value, tags);

        let result = {
            self.write_pool.prep_exec(
                        "INSERT INTO items (type, name, value, tags, wallet_id) VALUE (:type, :name, :value, :tags, :wallet_id)",
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
                    warn!("MySQL Error while executing query. Err Code: {}", err.code);
                    match err.code {
                        1062 => return ErrorCode::ItemAlreadyExists,
                        3140 => return ErrorCode::InvalidStructure, // Invalid JSON
                        _ => return ErrorCode::IOError,
                    };
                },
                Err(err) => {
                    warn!("Unexpected Error while executing query. Err: {:?}", err);
                    return ErrorCode::IOError
                },
                Ok(result) => result,
        };

        trace!("Success Adding Record with the type: {}, id: {}", type_, id);

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
    ///  * `InvalidStructure` - Invalid structure of the JSON arguments -> options
    ///  * `ItemNotFound` - Record with the provided type and id does not exist in the DB
    ///  * `IOError` - Unexpected error occurred while communicating with the DB
    ///  * `InvalidState` - Invalid encoding of a provided/fetched string
    ///
    pub fn fetch_record(&self, type_: &str, id: &str, options: &str, record_handle_p: *mut i32) -> ErrorCode {

        trace!("Fetching Record -> type: {}, id: {}, options: {}", type_, id, options);

        let options: FetchOptions = check_result!(serde_json::from_str(options), ErrorCode::InvalidStructure);

        let record: Record;

        let query = format!(
            "SELECT {}, {} \
             FROM items i \
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

        let row = check_result!(check_option!(result.next(), ErrorCode::ItemNotFound), ErrorCode::IOError);

        // These 2 values cannot be NULL.
        let db_value: Vec<u8> = check_option!(row.get(0), ErrorCode::IOError);
        let tags: String = check_option!(row.get(1), ErrorCode::IOError);

        record = Record::new(
            check_result!(CString::new(id), ErrorCode::InvalidState),
            if options.retrieve_value {Some(db_value)} else {None},
            if options.retrieve_tags {Some(check_result!(CString::new(tags), ErrorCode::InvalidState))} else {None},
            Some(check_result!(CString::new(type_), ErrorCode::InvalidState))
        );

        let record_handle = self.records.insert(record);

        unsafe { *record_handle_p = record_handle; }

        trace!("Success Fetching Record with the type: {}, id: {}. Record Handle: {}", type_, id, record_handle);

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

        trace!("Getting Record -> record_handle: {}", record_handle);

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
    ///  * `NotFoundError` - Record with the provided type and id does not exist in the DB
    ///  * `IOError` - Unexpected error occurred while communicating with the DB
    ///
    pub fn delete_record(&self, type_: &str, id: &str) -> ErrorCode {

        trace!("Deleting record -> type: {}, id: {}", type_, id);

        let result: QueryResult = check_result!(
            self.write_pool.prep_exec(
                "DELETE FROM items WHERE type = :type AND name = :name AND wallet_id = :wallet_id",
                params! {
                    "type" => type_,
                    "name" => id,
                    "wallet_id" => self.wallet_id,
                }
            ),
            ErrorCode::IOError
        );

        if result.affected_rows() != 1 {
            warn!("Trying to delete a non existent record, type: {}, id: {}", type_, id);
            return ErrorCode::ItemNotFound;
        }

        trace!("Success Deleting Record with the type: {}, id: {}", type_, id);

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
    ///  * `NotFoundError` - Record with the provided type and id does not exist in the DB
    ///  * `IOError` - Unexpected error occurred while communicating with the DB
    ///
    pub fn update_record_value(&self, type_: &str, id: &str, value: &Vec<u8>) -> ErrorCode {

        trace!("Updating Record Value -> type: {}, id: {}, value: {:?}", type_, id, value);

        let result: QueryResult = check_result!(
            self.write_pool.prep_exec(
                "UPDATE items SET value = :value WHERE type = :type AND name = :name AND wallet_id = :wallet_id",
                    params!{
                        "value" => value,
                        "type" => type_,
                        "name" => id,
                        "wallet_id" => self.wallet_id
                    }
            ),
            ErrorCode::IOError
        );

        if result.affected_rows() != 1 {
            warn!("Trying to update value of a non existent record, type: {}, id: {}", type_, id);
            return ErrorCode::ItemNotFound;
        }

        trace!("Success Updating Value of a record with the type: {}, id: {}", type_, id);

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
    ///  * `NotFoundError` - Record with the provided type and id does not exist in the DB
    ///  * `InvalidStructure` - Invalid structure of the JSON arguments -> tags
    ///  * `IOError` - Unexpected error occurred while communicating with the DB
    ///
    pub fn add_record_tags(&self, type_: &str, id: &str, tags: &HashMap<String, serde_json::Value>) -> ErrorCode {

        trace!("Adding Record Tags -> type: {}, id: {}, tags:{:?}", type_, id, tags);

        if tags.is_empty() {
            trace!("No tags to add. Checking if record exists...");
            return self._check_if_record_exists(type_, id);
        }

        let mut tag_name_value_paths: Vec<String> = Vec::new();

        for (tag_name, tag_value) in tags {
            let tag_name_path = format!(r#"'$."{}"', {}"#, tag_name, tag_value);
            tag_name_value_paths.push(tag_name_path);
        }

        let tag_name_value_paths = tag_name_value_paths.join(",");

        trace!("JSON Path argument given to JSON_SET is: {}", tag_name_value_paths);

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
            Err(Error::MySqlError(err)) => {
                warn!("MySQL Error while executing query. Err Code: {}", err.code);
                match err.code {
                    1064 => return ErrorCode::InvalidStructure, // Invalid JSON
                    _ => return ErrorCode::IOError,
                };
            },
            Err(err) => {
                    warn!("Unexpected Error while executing query. Err: {:?}", err);
                    return ErrorCode::IOError
            },
            Ok(result) => result,
        };

        if result.affected_rows() != 1 {
            warn!("Trying to add tags to a non existent record, type: {}, id: {}", type_, id);
            return ErrorCode::ItemNotFound;
        }

        trace!("Success Adding Tags for the record with the type: {}, id: {}", type_, id);

        ErrorCode::Success
    }

    ///
    /// Updates tags of a record identified by type and id.
    ///
    /// `WARNING`: Performs a destructive update by replacing old JSON doc with a new one.
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
    ///  * `NotFoundError` - Record with the provided type and id does not exist in the DB
    ///  * `InvalidStructure` - Invalid structure of the JSON arguments -> tags
    ///  * `IOError` - Unexpected error occurred while communicating with the DB
    ///
    pub fn update_record_tags(&self, type_: &str, id: &str, tags: &str) -> ErrorCode {

        trace!("Updating Record Tags -> type: {}, id: {}, tags: {}", type_, id, tags);

        let result = {
            self.write_pool.prep_exec(
                        "UPDATE items SET tags = :tags WHERE type = :type AND name = :name AND wallet_id = :wallet_id",
                        params!{
                            "tags" => tags,
                            "type" => type_,
                            "name" => id,
                            "wallet_id" => self.wallet_id
                        }
                )
        };

        let result = match result {
            Err(Error::MySqlError(err)) => {
                warn!("MySQL Error while executing query. Err Code: {}", err.code);
                match err.code {
                    3140 => return ErrorCode::InvalidStructure, // Invalid JSON
                    _ => return ErrorCode::IOError,
                };
            },
            Err(err) => {
                    warn!("Unexpected Error while executing query. Err: {:?}", err);
                    return ErrorCode::IOError
            },
            Ok(result) => result,
        };

        if result.affected_rows() != 1 {
            warn!("Trying to update tags of a non existent record, type: {}, id: {}", type_, id);
            return ErrorCode::ItemNotFound;
        }

        trace!("Success Updating Tags for the record with the type: {}, id: {}", type_, id);

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
    ///  * `InvalidStructure` - Invalid structure of the JSON arguments -> tag_names
    ///  * `NotFoundError` - Record with the provided type and id does not exist in the DB
    ///  * `IOError` - Unexpected error occurred while communicating with the DB
    ///
    pub fn delete_record_tags(&self, type_: &str, id: &str, tag_names: &Vec<String>) -> ErrorCode {

        trace!("Deleting Record Tags -> type: {}, id: {}, tag_names: {:?}", type_, id, tag_names);

        if tag_names.is_empty() {
            trace!("No tags to delete. Checking if record exists...");
            return self._check_if_record_exists(type_, id);
        }

        let mut tag_name_paths: Vec<String> = Vec::new();

        for tag_name in tag_names {
            let tag_name_path = format!(r#"'$."{}"'"#, tag_name);
            tag_name_paths.push(tag_name_path);
        }

        let tag_name_paths = tag_name_paths.join(",");

        trace!("JSON Path argument given to JSON_REMOVE is: {}", tag_name_paths);

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
            Err(Error::MySqlError(err)) => {
                warn!("MySQL Error while executing query. Err Code: {}", err.code);
                match err.code {
                    1064 => return ErrorCode::InvalidStructure, // Invalid JSON
                    _ => return ErrorCode::IOError,
                };
            },
            Err(err) => {
                    warn!("Unexpected Error while executing query. Err: {:?}", err);
                    return ErrorCode::IOError
            },
            Ok(result) => result,
        };

        if result.affected_rows() != 1 {
            warn!("Trying to delete tags of a non existent record, type: {}, id: {}", type_, id);
            return ErrorCode::ItemNotFound;
        }

        trace!("Success Deleting Tags for the record with the type: {}, id: {}", type_, id);

        ErrorCode::Success
    }

    ///
    /// Gets wallet metadata.
    ///
    /// # Returns
    ///
    ///  * `Result<(Arc<CString>, i32), ErrorCode>` - metadata content or ErrorCode
    ///
    /// # ErrorCodes
    ///
    ///  * `Success` - Execution successful
    ///  * `InvalidState` - Invalid encoding of a provided/fetched string
    ///  * `ItemNotFound` - Record with the provided type and id does not exist in the DB
    ///  * `IOError` - Unexpected error occurred while communicating with the DB
    ///
    pub fn get_metadata(&self) -> Result<(Arc<CString>, i32), ErrorCode> {

        trace!("Getting Wallet Metadata");

        let mut result: QueryResult = check_result!(
            self.read_pool.prep_exec(
                "SELECT metadata FROM wallets WHERE id = :wallet_id",
                params! {
                    "wallet_id" => self.wallet_id,
                }
            ),
            Err(ErrorCode::IOError)
        );

        let row = check_result!(check_option!(result.next(), Err(ErrorCode::ItemNotFound)), Err(ErrorCode::IOError));
        let metadata: String = check_option!(row.get(0), Err(ErrorCode::IOError));
        let metadata = check_result!(CString::new(metadata), Err(ErrorCode::InvalidState));

        let handle = self.metadata.insert(metadata);
        let metadata = check_option!(self.metadata.get(handle), Err(ErrorCode::ItemNotFound));

        trace!("Success Getting Wallet Metadata");

        Ok((metadata, handle))
    }

    ///
    /// Sets wallet metadata.
    ///
    /// # Arguments
    ///
    ///  * `metadata` - metadata to be stored
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
    pub fn set_metadata(&self, metadata: &str) -> ErrorCode {

        trace!("Setting Wallet Metadata");

        check_result!(
            self.write_pool.prep_exec(
                "UPDATE wallets SET metadata = :metadata WHERE id = :wallet_id",
                params! {
                    "wallet_id" => self.wallet_id,
                    "metadata" => metadata,
                }
            ),
            ErrorCode::IOError
        );

        trace!("Success Setting Wallet Metadata");

        ErrorCode::Success
    }

    ///
    /// Removes a metadata handle, thus removing the referenced object from memory.
    ///
    /// # Arguments
    ///
    ///  * `metadata_handle` - unique identifier of wallets metadata
    ///
    /// # Returns
    ///
    ///  * `ErrorCode`
    ///
    /// # ErrorCodes
    ///
    ///  * `Success` - Execution successful
    ///  * `InvalidState` - Provided metadata handle does not exist
    ///
    pub fn free_metadata(&self, metadata_handle: i32) -> ErrorCode {

        trace!("Freeing Wallet Metadata -> metadata_handle: {}", metadata_handle);

        if self.metadata.remove(metadata_handle) {
            trace!("Success Freeing Wallet Metadata with the metadata_handle: {}", metadata_handle);
            ErrorCode::Success
        }
        else {
            warn!("Trying to free non existent Wallet Metadata, metadata_handle: {}", metadata_handle);
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

        trace!("Searching Records -> type: {}, query_json: {}, options: {}", type_, query_json, options_json);

        let search_options: SearchOptions = check_result!(serde_json::from_str(options_json), ErrorCode::InvalidStructure);

        let wql = check_result!(query_translator::parse_from_json(&query_json), ErrorCode::InvalidStructure);

        let total_count = if search_options.retrieve_total_count {

            trace!("Searching Records -> retrieve_total_count branch");

            let (query, arguments) = check_result!(query_translator::wql_to_sql_count(self.wallet_id, type_, &wql), ErrorCode::InvalidStructure);
            let mut result: QueryResult = check_result!(
                self.read_pool.prep_exec(query, arguments),
                ErrorCode::IOError
            );

            let row = check_result!(check_option!(result.next(), ErrorCode::IOError), ErrorCode::IOError);
            let count: usize = check_option!(row.get(0), ErrorCode::IOError);

            Some(count)

        } else {None};

        let records_result = if search_options.retrieve_records {

            trace!("Searching Records -> retrieve_records branch");

            let (query, arguments) = check_result!(query_translator::wql_to_sql(self.wallet_id, type_, &wql, &search_options), ErrorCode::InvalidStructure);

            let search_result: QueryResult = check_result!(
                self.read_pool.prep_exec(query, arguments),
                ErrorCode::IOError
            );

            Some(search_result)
        } else {None};

        let search_handle = self.searches.insert(Search::new(records_result, total_count));

        unsafe { *search_handle_p = search_handle; }

        trace!("Success Searching Records with the type: {}, query_json: {}. Search Handle: {}", type_, query_json, search_handle);

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

        trace!("Searching All Records");

         let search_result: QueryResult = check_result!(
            self.read_pool.prep_exec(
                "SELECT type, name, value, tags FROM items WHERE wallet_id = :wallet_id",
                params! {
                    "wallet_id" => self.wallet_id,
                }
            ),
            ErrorCode::IOError
        );

        let search_handle = self.searches.insert(Search::new(Some(search_result), None));

        unsafe { *search_handle_p = search_handle; }

        trace!("Success Searching All Records. Search Handle: {}", search_handle);

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
    ///  * `ItemNotFound` - Result set exhausted, no more records to fetch
    ///
    pub fn fetch_search_next_record(&self, search_handle: i32, record_handle_p: *mut i32) -> ErrorCode {

        trace!("Fetching Search Next Record -> search_handle: {}", search_handle);

        let search = check_option!(self.searches.get(search_handle), ErrorCode::InvalidState);

        match search.search_result {
            None => {
                warn!("Trying to fetch results for a search that wasn't meant to retrieve any records, search_handle: {}", search_handle);
                ErrorCode::InvalidState
            },
            Some(ref search_result) => {
                let mut search_result = check_result!(search_result.write(), ErrorCode::IOError);

                let next_result = check_option!(search_result.next(), ErrorCode::ItemNotFound);

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

                trace!("Success Fetching Search Next Record with the search_handle: {}. Record Handle: {}", search_handle, record_handle);

                ErrorCode::Success
            }
        }
    }

    ///
    /// Fetches a total count of search result set
    ///
    /// # Arguments
    ///
    ///  * `search_handle` - unique identifier of a search request
    ///  * `total_count_p` - output param - total count
    ///
    /// # Returns
    ///
    ///  * `ErrorCode`
    ///
    /// # ErrorCodes
    ///
    ///  * `Success` - Execution successful
    ///  * `InvalidState` - Provided search handle does not exist, or parsing of data has gone wrong
    ///
    pub fn get_search_total_count(&self, search_handle: i32, total_count_p: *mut usize) -> ErrorCode {

        trace!("Getting Search Total Count -> search_handle: {}", search_handle);

        let search = check_option!(self.searches.get(search_handle), ErrorCode::InvalidState);

        match search.total_count {
            None => {
                warn!("Trying to get total count for a search that wasn't meant to retrieve total count, search_handle: {}", search_handle);
                ErrorCode::InvalidState
            },
            Some(total_count) => {
                unsafe { *total_count_p = total_count };
                trace!("Success Getting Search Total Count with the search_handle: {}. Total Count: {}", search_handle, total_count);
                ErrorCode::Success
            }
        }
    }

    ///
    /// Helper method that checks if a record exists.
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
    ///  * `IOError` - Unexpected error occurred while communicating with the DB
    ///  * `ItemNotFound` - Record with provided `type_` and `id` does not exists
    ///
    fn _check_if_record_exists(&self, type_: &str, id: &str) -> ErrorCode {
        let mut result = check_result!(
            self.read_pool.prep_exec(
                "SELECT 1 \
                FROM items \
                WHERE type = :type \
                    AND name = :name \
                    AND wallet_id = :wallet_id",
                params!{
                    "type" => type_,
                    "name" => id,
                    "wallet_id" => self.wallet_id
                }),
            ErrorCode::IOError);

        check_result!(check_option!(result.next(), ErrorCode::ItemNotFound), ErrorCode::IOError);

        return ErrorCode::Success;
    }
}

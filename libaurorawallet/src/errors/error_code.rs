#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(i32)]
pub enum ErrorCode {
    /// <summary>
    /// Call succeeded.
    /// </summary>
    Success = 0,

    /// <summary>
    /// Invalid library state was detected in runtime. It signals library bug
    /// </summary>
    InvalidState = 112,

    /// <summary>
    /// Object (json, config, and etc...) passed by library caller has invalid structure
    /// </summary>
    InvalidStructure = 113,

    /// <summary>
    /// IO Error in communicating with the DB - broken query, server unavailable...
    /// </summary>
    IOError = 114,

    /// <summary>
    /// Attempt to create wallet with name used for another exists wallet
    /// </summary>
    WalletAlreadyExistsError = 203,

    /// <summary>
    /// Requested entity id isn't present in wallet.
    /// </summary>
    WalletNotFoundError = 204,

    /// <summary>
    /// Entity already exists in the wallet.
    /// </summary>
    RecordAlreadyExists = 208,
}

macro_rules! check_result {
    ($r: expr, $e: expr) => {
        match $r {
            Err(_err) => return $e,
            Ok(x) => x
        }
    }
}

macro_rules! check_option {
    ($o: expr, $e: expr) => {
        match $o {
            None => return $e,
            Some(x) => x
        }
    }
}

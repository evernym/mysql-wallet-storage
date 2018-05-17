#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(i32)]
pub enum ErrorCode {
    Success = 0,
    InvalidState = 112,
    InvalidStructure = 113,
    IOError = 114,
    WalletNotFoundError = 204,
    RecordAlreadyExists = 208,
    WalletItemNotFound = 212
}

macro_rules! check_result {
    ($r: expr, $e: expr) => {
        match $r {
            Err(_) => { return $e },
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

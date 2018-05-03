

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(i32)]
pub enum ErrorCode {
    Success = 0,
    InvalidStorageHandle = 1,
    InvalidSearchHandle = 2,
    InvalidRecordHandle = 3,
}
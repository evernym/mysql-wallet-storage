extern crate libc;

extern crate serde_json;

#[macro_use]
extern crate mysql;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate lazy_static;

mod utils;

#[macro_use]
pub mod errors;

pub mod api;

mod aurora_storage;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
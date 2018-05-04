extern crate serde_json;
extern crate libc;

#[macro_use]
extern crate mysql;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate lazy_static;

#[macro_use]
mod utils;

pub mod api;
mod aurora_storage;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

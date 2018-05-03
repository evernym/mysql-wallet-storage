extern crate serde_json;
extern crate libc;
extern crate mysql;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate lazy_static;

pub mod api;
mod aurora_storage;
mod utils;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

extern crate libc;

extern crate serde_json;

#[macro_use]
extern crate mysql;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate lazy_static;

pub mod utils;

#[macro_use]
pub mod errors;

pub mod api;

mod aurora_storage;

mod libindy;

use std::ffi::CString;
use errors::error_code::ErrorCode;
use std::sync::mpsc::channel;
use utils::callbacks;

#[no_mangle]
pub extern fn aurora_storage_init() -> ErrorCode {
    let storage_name = CString::new("aurora").unwrap();

    let (sender, receiver) = channel();

    let closure: Box<FnMut(ErrorCode) + Send> = Box::new(move |err| {
        sender.send(err).unwrap();
    });

    let (cmd_handle, cb) = callbacks::closure_to_cb_ec(closure);

    unsafe {
        libindy::indy_register_wallet_storage(
            cmd_handle,
            storage_name.as_ptr(),
            Some(api::create_storage),
            Some(api::open_storage),
            Some(api::close_storage),
            Some(api::delete_storage),
            Some(api::add_record),
            Some(api::update_record_value),
            Some(api::update_record_tags),
            Some(api::add_record_tags),
            Some(api::delete_record_tags),
            Some(api::delete_record),
            Some(api::get_record),
            Some(api::get_record_id),
            Some(api::get_record_type),
            Some(api::get_record_value),
            Some(api::get_record_tags),
            Some(api::free_record),
            Some(api::get_metadata),
            Some(api::set_metadata),
            Some(api::free_metadata),
            Some(api::search_records),
            Some(api::search_all_records),
            Some(api::get_search_total_count),
            Some(api::fetch_search_next_record),
            Some(api::free_search),
            cb,
        );
    }

    receiver.recv().unwrap()
}




#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

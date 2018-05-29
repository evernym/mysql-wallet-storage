use errors::error_code::ErrorCode;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use std::sync::Mutex;
use std::collections::HashMap;


type EcClosure = Box<FnMut(ErrorCode) + Send>;
type EcCallback = Option<extern fn(command_handle: i32, err: ErrorCode)>;


lazy_static! {
    static ref IDS_COUNTER: AtomicUsize = ATOMIC_USIZE_INIT; //TODO use AtomicI32
}

pub fn get_next_id() -> i32 {
        (IDS_COUNTER.fetch_add(1, Ordering::SeqCst) + 1) as i32
    }

pub fn closure_to_cb_ec(closure: EcClosure) -> (i32, EcCallback) {
    lazy_static! {
       static ref CALLBACKS: Mutex<HashMap<i32, EcClosure>> = Default::default();
    }

    extern "C" fn _callback(command_handle: i32, err: ErrorCode) {
        let mut callbacks = CALLBACKS.lock().unwrap();
        let mut cb = callbacks.remove(&command_handle).unwrap();
        cb(err)
    }

    let command_handle = get_next_id();
    let mut callbacks = CALLBACKS.lock().unwrap();
    callbacks.insert(command_handle, closure);

    (command_handle, Some(_callback))
}
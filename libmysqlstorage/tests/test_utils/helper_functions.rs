extern crate rand;
extern crate serde;
extern crate serde_json;

use std::collections::HashMap;

use self::rand::{thread_rng, Rng};
use self::rand::distributions::{Range, IndependentSample};

pub fn random_string(char_num: usize) -> String {
    thread_rng().gen_ascii_chars().take(char_num).collect()
}

pub fn random_name() -> String {
    random_string(10)
}

pub fn get_random_record_value() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let range = Range::new(0, 255);
    let vals: Vec<u8> = (0..300).map(|_| range.ind_sample(&mut rng)).collect();
    vals
}

pub fn get_hash_map_from_json_string(json_string: &str) -> HashMap<String, String>{
    let map: HashMap<String, String> = serde_json::from_str(json_string).unwrap();
    map
}
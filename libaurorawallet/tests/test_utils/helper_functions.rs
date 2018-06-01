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

pub fn generate_predefinied_tags(tags_count: u64) -> String{
    let mut tags_list = HashMap::new();
    for t in 0..tags_count {
        let key: String;
        let value: String = format!("value_{}", t+1);
        if t % 2 == 0 {
             key = format!("tag_{}", t+1);
        }
        else {
            key = format!("~tag_{}", t+1);
        }
        tags_list.insert(key, value);
    }
    let tags = serde_json::to_string(&tags_list).unwrap();
    tags
}

pub fn get_predefined_tag_names(tags_count: u64) -> Vec<String>{
    let mut tag_names: Vec<String> = Vec::new();

    for t in 0..tags_count {
        let tag_name: String;
        if t % 2 == 0 {
             tag_name = format!("tag_{}", t+1);
        }
        else {
            tag_name = format!("~tag_{}", t+1);
        }
        tag_names.push(tag_name);
    }
    tag_names
}
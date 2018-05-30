extern crate rand;
use self::rand::{thread_rng, Rng};

pub fn random_string(char_num: usize) -> String {
    thread_rng().gen_ascii_chars().take(char_num).collect()
}

pub fn random_name() -> String {
    random_string(10)
}

extern crate amf;

use std::io;
use amf::{Value, Version};

fn main() {
    let mut input = io::stdin();
    let amf0_value = Value::read_from(&mut input, Version::Amf0).unwrap();
    println!("VALUE: {:?}", amf0_value);
}

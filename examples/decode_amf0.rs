extern crate amf;

use amf::{Value, Version};
use std::io;

fn main() {
    let mut input = io::stdin();
    let amf0_value = Value::read_from(&mut input, Version::Amf0).unwrap();
    println!("VALUE: {:?}", amf0_value);
}

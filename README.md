amf
===

[![amf](https://img.shields.io/crates/v/amf.svg)](https://crates.io/crates/amf)
[![Documentation](https://docs.rs/amf/badge.svg)](https://docs.rs/amf)
[![Actions Status](https://github.com/sile/amf/workflows/CI/badge.svg)](https://github.com/sile/amf/actions)
[![Coverage Status](https://coveralls.io/repos/github/sile/amf/badge.svg?branch=master)](https://coveralls.io/github/sile/amf?branch=master)
![License](https://img.shields.io/crates/l/amf)

A Rust Implementation of AMF (Action Media Format).


Documentation
-------------

See [RustDoc Documentation](https://docs.rs/amf/).

Example
-------

Following code decodes a AMF0 encoded value read from the standard input:

```rust
// file: examples/decode_amf0.rs
extern crate amf;

use std::io;
use amf::{Value, Version};

fn main() {
    let mut input = io::stdin();
    let amf0_value = Value::read_from(&mut input, Version::Amf0).unwrap();
    println!("VALUE: {:?}", amf0_value);
}
```

An execution result:

```bash
$ cat src/testdata/amf0-number.bin | cargo run --example decode_amf0
VALUE: Amf0(Number(3.5))
```

References
----------

- [AMF0 Specification](http://download.macromedia.com/pub/labs/amf/amf0_spec_121207.pdf)
- [AMF3 Specification](https://www.adobe.com/content/dam/acom/en/devnet/pdf/amf-file-format-spec.pdf)
- [Action Message Format - Wikipedia](https://en.wikipedia.org/wiki/Action_Message_Format)

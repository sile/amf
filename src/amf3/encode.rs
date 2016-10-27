use std::io;
use std::time;
use byteorder::BigEndian;
use byteorder::WriteBytesExt;

use Pair;
use super::Value;
use super::marker;

#[derive(Debug)]
pub struct Encoder<W> {
    inner: W,
}
impl<W> Encoder<W> {
    pub fn into_inner(self) -> W {
        self.inner
    }
}
impl<W> Encoder<W>
    where W: io::Write
{
    pub fn new(inner: W) -> Self {
        Encoder { inner: inner }
    }
    pub fn encode(&mut self, value: &Value) -> io::Result<()> {
        match *value {
            Value::Undefined => self.encode_undefined(),
            Value::Null => self.encode_null(),
            Value::Boolean(x) => self.encode_boolean(x),
            Value::Integer(x) => self.encode_integer(x),
            Value::Double(x) => self.encode_double(x),
            Value::String(ref x) => self.encode_string(x),
            Value::XmlDocument(ref x) => self.encode_xml_document(x),
            Value::Date { unix_time } => self.encode_date(unix_time),
            Value::Array { ref assoc_entries, ref dense_entries } => {
                self.encode_array(assoc_entries, dense_entries)
            }
            Value::Object { ref class_name, sealed_count, ref entries } => {
                self.encode_object(class_name, sealed_count, entries)
            }
            Value::Xml(ref x) => self.encode_xml(x),
            Value::ByteArray(ref x) => self.encode_byte_array(x),
            Value::IntVector { is_fixed, ref entries } => self.encode_int_vector(is_fixed, entries),
            Value::UintVector { is_fixed, ref entries } => {
                self.encode_uint_vector(is_fixed, entries)
            }
            Value::DoubleVector { is_fixed, ref entries } => {
                self.encode_double_vector(is_fixed, entries)
            }
            Value::ObjectVector { ref class_name, is_fixed, ref entries } => {
                self.encode_object_vector(class_name, is_fixed, entries)
            }
            Value::Dictionary { is_weak, ref entries } => self.encode_dictionary(is_weak, entries),
        }
    }

    fn encode_undefined(&mut self) -> io::Result<()> {
        try!(self.inner.write_u8(marker::UNDEFINED));
        Ok(())
    }
    fn encode_null(&mut self) -> io::Result<()> {
        try!(self.inner.write_u8(marker::NULL));
        Ok(())
    }
    fn encode_boolean(&mut self, b: bool) -> io::Result<()> {
        if b {
            try!(self.inner.write_u8(marker::TRUE));
        } else {
            try!(self.inner.write_u8(marker::FALSE));
        }
        Ok(())
    }
    fn encode_integer(&mut self, i: i32) -> io::Result<()> {
        try!(self.inner.write_u8(marker::INTEGER));
        try!(self.inner.write_i32::<BigEndian>(i));
        Ok(())
    }
    fn encode_double(&mut self, d: f64) -> io::Result<()> {
        try!(self.inner.write_u8(marker::DOUBLE));
        try!(self.inner.write_f64::<BigEndian>(d));
        Ok(())
    }
    fn encode_string(&mut self, s: &str) -> io::Result<()> {
        try!(self.inner.write_u8(marker::STRING));
        try!(self.encode_utf8(s));
        Ok(())
    }
    fn encode_xml_document(&mut self, xml: &str) -> io::Result<()> {
        try!(self.inner.write_u8(marker::XML_DOC));
        try!(self.encode_utf8(xml));
        Ok(())
    }
    fn encode_date(&mut self, unix_time: time::Duration) -> io::Result<()> {
        let millis = unix_time.as_secs() * 1000 + (unix_time.subsec_nanos() as u64) / 1000_000;
        try!(self.inner.write_u8(marker::DATE));
        try!(self.inner.write_f64::<BigEndian>(millis as f64));
        Ok(())
    }
    fn encode_array(&mut self, assoc: &[Pair<String, Value>], dense: &[Value]) -> io::Result<()> {
        try!(self.inner.write_u8(marker::ARRAY));
        try!(self.encode_size(dense.len()));
        try!(self.encode_pairs(assoc));
        try!(dense.iter().map(|v| self.encode(v)).collect::<io::Result<Vec<_>>>());
        Ok(())
    }
    fn encode_object(&mut self,
                     class_name: &Option<String>,
                     sealed_count: usize,
                     entries: &[Pair<String, Value>])
                     -> io::Result<()> {
        try!(self.encode_trait(class_name, sealed_count, entries));
        for e in entries.iter().take(sealed_count) {
            try!(self.encode(&e.value));
        }
        try!(self.encode_pairs(&entries[sealed_count..]));
        Ok(())
    }
    fn encode_xml(&mut self, xml: &str) -> io::Result<()> {
        try!(self.inner.write_u8(marker::XML));
        try!(self.encode_utf8(xml));
        Ok(())
    }
    fn encode_byte_array(&mut self, bytes: &[u8]) -> io::Result<()> {
        try!(self.inner.write_u8(marker::BYTE_ARRAY));
        try!(self.encode_size(bytes.len()));
        try!(self.inner.write_all(bytes));
        Ok(())
    }
    fn encode_int_vector(&mut self, is_fixed: bool, vec: &[i32]) -> io::Result<()> {
        try!(self.inner.write_u8(marker::VECTOR_INT));
        try!(self.encode_size(vec.len()));
        try!(self.inner.write_u8(is_fixed as u8));
        for &x in vec {
            try!(self.inner.write_i32::<BigEndian>(x));
        }
        Ok(())
    }
    fn encode_uint_vector(&mut self, is_fixed: bool, vec: &[u32]) -> io::Result<()> {
        try!(self.inner.write_u8(marker::VECTOR_UINT));
        try!(self.encode_size(vec.len()));
        try!(self.inner.write_u8(is_fixed as u8));
        for &x in vec {
            try!(self.inner.write_u32::<BigEndian>(x));
        }
        Ok(())
    }
    fn encode_double_vector(&mut self, is_fixed: bool, vec: &[f64]) -> io::Result<()> {
        try!(self.inner.write_u8(marker::VECTOR_DOUBLE));
        try!(self.encode_size(vec.len()));
        try!(self.inner.write_u8(is_fixed as u8));
        for &x in vec {
            try!(self.inner.write_f64::<BigEndian>(x));
        }
        Ok(())
    }
    fn encode_object_vector(&mut self,
                            class_name: &Option<String>,
                            is_fixed: bool,
                            vec: &[Value])
                            -> io::Result<()> {
        try!(self.inner.write_u8(marker::VECTOR_OBJECT));
        try!(self.encode_size(vec.len()));
        try!(self.inner.write_u8(is_fixed as u8));
        try!(self.encode_utf8(class_name.as_ref().map_or("*", |s| &s)));
        for x in vec {
            try!(self.encode(x));
        }
        Ok(())
    }
    fn encode_dictionary(&mut self,
                         is_weak: bool,
                         entries: &[Pair<Value, Value>])
                         -> io::Result<()> {
        try!(self.inner.write_u8(marker::DICTIONARY));
        try!(self.encode_size(entries.len()));
        try!(self.inner.write_u8(is_weak as u8));
        for e in entries {
            try!(self.encode(&e.key));
            try!(self.encode(&e.value));
        }
        Ok(())
    }

    fn encode_trait(&mut self,
                    class_name: &Option<String>,
                    sealed_count: usize,
                    entries: &[Pair<String, Value>])
                    -> io::Result<()> {
        assert!(sealed_count <= entries.len());
        let is_externalizable = false as usize;
        let is_dynamic = (sealed_count < entries.len()) as usize;
        let u28 = (entries.len() << 2) | (is_dynamic << 1) | is_externalizable;
        try!(self.encode_size(u28));

        let class_name = class_name.as_ref().map_or("", |s| &s);
        try!(self.encode_utf8(class_name));
        for e in entries.iter().take(sealed_count) {
            try!(self.encode_utf8(&e.key));
        }
        Ok(())
    }
    fn encode_size(&mut self, size: usize) -> io::Result<()> {
        assert!(size < (1 << 28));
        let not_reference = 1;
        self.encode_u29(((size << 1) | not_reference) as u32)
    }
    fn encode_u29(&mut self, u29: u32) -> io::Result<()> {
        if u29 < 0x80 {
            try!(self.inner.write_u8(u29 as u8));
        } else if u29 < 0x4000 {
            let b1 = ((u29 >> 0) & 0b0111_1111) as u8;
            let b2 = ((u29 >> 7) | 0b1000_0000) as u8;
            for b in &[b2, b1] {
                try!(self.inner.write_u8(*b));
            }
        } else if u29 < 0x20_0000 {
            let b1 = ((u29 >> 00) & 0b0111_1111) as u8;
            let b2 = ((u29 >> 07) | 0b1000_0000) as u8;
            let b3 = ((u29 >> 14) | 0b1000_0000) as u8;
            for b in &[b3, b2, b1] {
                try!(self.inner.write_u8(*b));
            }
        } else if u29 < 0x4000_0000 {
            let b1 = ((u29 >> 00) & 0b1111_1111) as u8;
            let b2 = ((u29 >> 08) | 0b1000_0000) as u8;
            let b3 = ((u29 >> 15) | 0b1000_0000) as u8;
            let b4 = ((u29 >> 22) | 0b1000_0000) as u8;
            for b in &[b4, b3, b2, b1] {
                try!(self.inner.write_u8(*b));
            }
        } else {
            panic!("Too large number: {}", u29);
        }
        Ok(())

    }
    fn encode_utf8(&mut self, s: &str) -> io::Result<()> {
        try!(self.encode_size(s.len()));
        try!(self.inner.write_all(s.as_bytes()));
        Ok(())
    }
    fn encode_pairs(&mut self, pairs: &[Pair<String, Value>]) -> io::Result<()> {
        for p in pairs {
            try!(self.encode_utf8(&p.key));
            try!(self.encode(&p.value));
        }
        try!(self.encode_utf8(""));
        Ok(())
    }
}

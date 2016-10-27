use std::io;
use std::time;
use byteorder::WriteBytesExt;
use byteorder::BigEndian;

use Pair;
use amf3;
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
            Value::Number(x) => self.encode_number(x),
            Value::Boolean(x) => self.encode_boolean(x),
            Value::String(ref x) => self.encode_string(x),
            Value::Object { ref class_name, ref entries } => {
                self.encode_object(class_name, entries)
            }
            Value::Null => self.encode_null(),
            Value::Undefined => self.encode_undefined(),
            Value::EcmaArray { ref entries } => self.encode_ecma_array(entries),
            Value::Array { ref entries } => self.encode_strict_array(entries),
            Value::Date { unix_time } => self.encode_date(unix_time),
            Value::XmlDocument(ref x) => self.encode_xml_document(x),
            Value::AvmPlus(ref x) => self.encode_avmplus(x),
        }
    }

    fn encode_number(&mut self, n: f64) -> io::Result<()> {
        try!(self.inner.write_u8(marker::NUMBER));
        try!(self.inner.write_f64::<BigEndian>(n));
        Ok(())
    }
    fn encode_boolean(&mut self, b: bool) -> io::Result<()> {
        try!(self.inner.write_u8(marker::BOOLEAN));
        try!(self.inner.write_u8(b as u8));
        Ok(())
    }
    fn encode_string(&mut self, s: &str) -> io::Result<()> {
        if s.len() > 0xFFFF {
            try!(self.inner.write_u8(marker::STRING));
            try!(self.write_str_u16(&s));
        } else {
            try!(self.inner.write_u8(marker::LONG_STRING));
            try!(self.write_str_u32(&s));
        }
        Ok(())
    }
    fn encode_object(&mut self,
                     class_name: &Option<String>,
                     entries: &[Pair<String, Value>])
                     -> io::Result<()> {
        assert!(entries.len() <= 0xFFFF_FFFF);
        if let Some(class_name) = class_name.as_ref() {
            try!(self.inner.write_u8(marker::TYPED_OBJECT));
            try!(self.write_str_u16(class_name));
        } else {
            try!(self.inner.write_u8(marker::OBJECT));
        }
        try!(self.inner.write_u32::<BigEndian>(entries.len() as u32));
        try!(self.encode_pairs(entries));
        Ok(())
    }
    fn encode_null(&mut self) -> io::Result<()> {
        try!(self.inner.write_u8(marker::NULL));
        Ok(())
    }
    fn encode_undefined(&mut self) -> io::Result<()> {
        try!(self.inner.write_u8(marker::UNDEFINED));
        Ok(())
    }
    fn encode_ecma_array(&mut self, entries: &[Pair<String, Value>]) -> io::Result<()> {
        assert!(entries.len() <= 0xFFFF_FFFF);
        try!(self.inner.write_u8(marker::ECMA_ARRAY));
        try!(self.inner.write_u32::<BigEndian>(entries.len() as u32));
        try!(self.encode_pairs(entries));
        Ok(())
    }
    fn encode_strict_array(&mut self, entries: &[Value]) -> io::Result<()> {
        assert!(entries.len() <= 0xFFFF_FFFF);
        try!(self.inner.write_u8(marker::STRICT_ARRAY));
        try!(self.inner.write_u32::<BigEndian>(entries.len() as u32));
        for e in entries {
            try!(self.encode(e));
        }
        Ok(())
    }
    fn encode_date(&mut self, unix_time: time::Duration) -> io::Result<()> {
        let millis = unix_time.as_secs() * 1000 + (unix_time.subsec_nanos() as u64) / 1000_000;

        try!(self.inner.write_u8(marker::DATE));
        try!(self.inner.write_f64::<BigEndian>(millis as f64));
        Ok(())
    }
    fn encode_xml_document(&mut self, xml: &str) -> io::Result<()> {
        try!(self.inner.write_u8(marker::XML_DOCUMENT));
        try!(self.write_str_u32(xml));
        Ok(())
    }
    fn encode_avmplus(&mut self, value: &amf3::Value) -> io::Result<()> {
        try!(self.inner.write_u8(marker::AVMPLUS_OBJECT));
        try!(amf3::Encoder::new(&mut self.inner).encode(value));
        Ok(())
    }

    fn write_str_u32(&mut self, s: &str) -> io::Result<()> {
        assert!(s.len() <= 0xFFFF_FFFF);
        try!(self.inner.write_u32::<BigEndian>(s.len() as u32));
        try!(self.inner.write_all(s.as_bytes()));
        Ok(())
    }
    fn write_str_u16(&mut self, s: &str) -> io::Result<()> {
        assert!(s.len() <= 0xFFFF);
        try!(self.inner.write_u16::<BigEndian>(s.len() as u16));
        try!(self.inner.write_all(s.as_bytes()));
        Ok(())
    }
    fn encode_pairs(&mut self, pairs: &[Pair<String, Value>]) -> io::Result<()> {
        for p in pairs {
            try!(self.write_str_u16(&p.key));
            try!(self.encode(&p.value));
        }
        try!(self.inner.write_u8(0));
        try!(self.inner.write_u8(marker::OBJECT_END_MARKER));
        Ok(())
    }
}

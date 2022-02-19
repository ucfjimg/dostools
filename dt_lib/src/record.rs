use crate::error::Error as MyError;
use std::str;

// record types
//
#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum RecordType {
    THeader = 0x80,
    Comment = 0x88,
    ExtDef = 0x8c,
    ModEnd = 0x8a,
    ModEnd32 = 0x8b,
    PubDef = 0x90,
    PubDef32 = 0x91,
    LNames = 0x96,
    SegDef = 0x98,
    SegDef32 = 0x99,
    GrpDef = 0x9a,
    LExtDef = 0xb4,
    LExtDef32 = 0xb5,
}

impl RecordType {
    pub fn is32(self) -> bool {
        ((self as u8) & 0x01) != 0
    }
}

impl TryFrom<u8> for RecordType {
    type Error = MyError;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == RecordType::THeader as u8 => Ok(RecordType::THeader),
            x if x == RecordType::Comment as u8 => Ok(RecordType::Comment),
            x if x == RecordType::ExtDef as u8 => Ok(RecordType::ExtDef),
            x if x == RecordType::ModEnd as u8 => Ok(RecordType::ModEnd),
            x if x == RecordType::ModEnd32 as u8 => Ok(RecordType::ModEnd32),
            x if x == RecordType::PubDef as u8 => Ok(RecordType::PubDef),
            x if x == RecordType::PubDef32 as u8 => Ok(RecordType::PubDef32),
            x if x == RecordType::LNames as u8 => Ok(RecordType::LNames),
            x if x == RecordType::SegDef as u8 => Ok(RecordType::SegDef),
            x if x == RecordType::SegDef32 as u8 => Ok(RecordType::SegDef32),
            x if x == RecordType::GrpDef as u8 => Ok(RecordType::GrpDef),
            x if x == RecordType::LExtDef as u8 => Ok(RecordType::LExtDef),
            x if x == RecordType::LExtDef32 as u8 => Ok(RecordType::LExtDef32),
            _ => Err(MyError::new(&format!("invalid record type ${:02x}", v)))
        }
    }
}

// A parsed record
//
#[derive(Debug)]
pub struct Record<'a> {
    pub rectype: Option<RecordType>,
    pub data: &'a [u8],
}

// Parser state for a record. This is a separate struct so
// the module user does not have to pass in the OmfRecord
// as mutable.
//
pub struct RecordParser<'a> {
    rec: &'a Record<'a>,
    next: usize,
}

impl<'a> RecordParser<'a> {
    // Create a parser from a record
    //
    pub fn new(rec: &'a Record<'a>) -> RecordParser<'a> {
        RecordParser{
            rec: rec,
            next: 0
        }
    }

    // Returns true if the entire record has been parsed
    //
    pub fn end(&self) -> bool {
        self.next >= self.rec.data.len()
    }

    // Return the next string in the record. Strings (in most
    // cases) have a lead byte containing the string length.
    // Strings are in ASCII.
    //
    // Strings which do not have a lead byte are handled
    // elsewhere
    //
    pub fn next_str(&mut self) -> Result<String, MyError> {
        if self.next == self.rec.data.len() {
            return Err(MyError::truncated())
        } 

        let len: usize = self.rec.data[self.next] as usize;
        if self.next + 1 + len > self.rec.data.len() {
            return Err(MyError::truncated())
        }

        let start = self.next + 1;
        let end = start + len;

        let s = match str::from_utf8(&self.rec.data[start..end]) {
            Ok(x) => x,
            _ => return Err(MyError::new("invalid string in record")),
        };

        self.next = end;

        Ok(s.to_string())
    }

    // extract a little endian unsigned integer. depending on
    // `is32`, the integer will be 2 or 4 bytes long in the 
    // record.
    //
    pub fn next_uint(&mut self, is32: bool) -> Result<u32, MyError> {
        let bytes: usize = if is32 { 4 } else { 2 };
        let mut value: u32 = 0;

        if self.next + bytes > self.rec.data.len() {
            return Err(MyError::truncated());
        }

        for i in 1..bytes+1 {
            let byte = self.rec.data[self.next + bytes - i] as u32;
            value = (value << 8) | byte;
        }

        self.next += bytes;

        Ok(value)
    }

    // extract an index. since most indices are small integers,
    // they are stored packed into one or two bytes as needed
    //
    pub fn next_index(&mut self) -> Result<usize, MyError> {
        let byte0 = self.next_byte()? as usize;

        if byte0 & 0x80 == 0 {
            return Ok(byte0);
        }

        let byte1 = self.next_byte()? as usize;

        Ok(((byte0 & 0x7f) << 8) | byte1)
    }

    // Return a string that takes up the rest of the record.
    // Unlike other strings, this one has no lead byte and
    // the length is therefore not limited to 256 bytes.
    //
    pub fn rest_str(&mut self) -> Result<String, MyError> {
        let start = self.next;
        let end = self.rec.data.len();

        if start > end {
            return Err(MyError::truncated())
        } 

        let s = match str::from_utf8(&self.rec.data[start..end]) {
            Ok(x) => x,
            _ => return Err(MyError::new("invalid string in record")),
        };

        self.next = end;

        Ok(s.to_string())
    }

    // Return the next byte in the record
    //
    pub fn next_byte(&mut self) -> Result<u8, MyError> {
        if self.next == self.rec.data.len() {
            return Err(MyError::truncated())
        } 
        
        let byte = self.rec.data[self.next];
        self.next += 1;

        Ok(byte)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn result_eq<T>(a: Result<T, MyError>, b: Result<T, MyError>) -> bool where
        T: std::cmp::PartialEq {
        match (a, b) {
            (Err(MyError{details: aerr}), Err(MyError{details:berr})) => aerr == berr,
            (Ok(aval), Ok(bval)) => aval == bval,
            _ => false,
        }
    }

    #[test]
    fn test_end_succeeds_on_empty_data() {
        let rec = Record{
            rectype: None,
            data: &vec![],
        };

        let parser = RecordParser::new(&rec);
        assert_eq!(parser.end(), true);
    }

    #[test]
    fn test_end_succeeds_on_data_left() {
        let rec = Record{
            rectype: None,
            data: &vec![0xfe],
        };

        let parser = RecordParser::new(&rec);
        assert_eq!(!parser.end(), true);
    }

    #[test]
    fn test_next_str_succeeds() {
        let rec = Record{
            rectype: None,
            data: &vec![3, 0x41, 0x42, 0x43],
        };

        let mut parser = RecordParser::new(&rec);
        assert!(result_eq(parser.next_str(), Ok("ABC".to_string())), "next_str didn't return correct string");
    }

    #[test]
    fn test_next_str_fails_on_empty() {
        let rec = Record{
            rectype: None,
            data: &vec![],
        };

        let mut parser = RecordParser::new(&rec);
        assert!(result_eq(parser.next_str(), Err(MyError::truncated())), "next_str didn't return trucated error");
    }

    #[test]
    fn test_next_str_leaves_valid_state() {
        let rec = Record{
            rectype: None,
            data: &vec![3, 0x41, 0x42, 0x43, 0xfe],
        };

        let mut parser = RecordParser::new(&rec);
        assert!(result_eq(parser.next_str(), Ok("ABC".to_string())), "next_str didn't return correct string");
        assert!(result_eq(parser.next_byte(), Ok(0xfe)), "next_str left parser in bad state");
    }

    #[test]
    fn test_next_str_fails_on_bounds() {
        let rec = Record{
            rectype: None,
            data: &vec![0x03, 0x41, 0x42],
        };

        let mut parser = RecordParser::new(&rec);
        assert!(result_eq(parser.next_str(), Err(MyError::truncated())), "next_str didn't return truncated error");
    }

    #[test]
    fn test_next_uint_32_succeeds() {
        let rec = Record{
            rectype: None,
            data: &vec![0x55, 0xaa, 0x34, 0x12],
        };

        let mut parser = RecordParser::new(&rec);
        assert!(result_eq(parser.next_uint(true), Ok(0x1234aa55)), "next_uint didn't return proper value");
    }

    #[test]
    fn test_next_uint_32_fails_on_bounds() {
        let rec = Record{
            rectype: None,
            data: &vec![0x55, 0xaa, 0x34],
        };

        let mut parser = RecordParser::new(&rec);
        assert!(result_eq(parser.next_uint(true), Err(MyError::truncated())), "next_uint didn't return truncated error");
    }

    #[test]
    fn test_next_uint_32_leaves_valid_state() {
        let rec = Record{
            rectype: None,
            data: &vec![0x55, 0xaa, 0x34, 0x12, 0x03],
        };

        let mut parser = RecordParser::new(&rec);
        assert!(result_eq(parser.next_uint(true), Ok(0x1234aa55)), "next_uint didn't return proper value");
        assert!(result_eq(parser.next_byte(), Ok(0x03)), "next_uint left parser in bad state");
    }

    #[test]
    fn test_next_uint_16_succeeds() {
        let rec = Record{
            rectype: None,
            data: &vec![0x55, 0xaa],
        };

        let mut parser = RecordParser::new(&rec);
        assert!(result_eq(parser.next_uint(false), Ok(0xaa55)), "next_uint didn't return proper value");
    }

    #[test]
    fn test_next_uint_16_fails_on_bounds() {
        let rec = Record{
            rectype: None,
            data: &vec![0x55],
        };

        let mut parser = RecordParser::new(&rec);
        assert!(result_eq(parser.next_uint(false), Err(MyError::truncated())), "next_uint didn't return truncated error");
    }

    #[test]
    fn test_next_uint_16_leaves_valid_state() {
        let rec = Record{
            rectype: None,
            data: &vec![0x55, 0xaa, 0x03],
        };

        let mut parser = RecordParser::new(&rec);
        assert!(result_eq(parser.next_uint(false), Ok(0xaa55)), "next_uint didn't return proper value");
        assert!(result_eq(parser.next_byte(), Ok(0x03)), "next_uint left parser in bad state");
    }

    #[test]
    fn test_next_small_index_succeeds() {
        let rec = Record{
            rectype: None,
            data: &vec![0x03],
        };

        let mut parser = RecordParser::new(&rec);
        assert!(result_eq(parser.next_index(), Ok(0x03)), "next_index didn't return proper value");
    }

    #[test]
    fn test_next_small_index_fails_on_bounds() {
        let rec = Record{
            rectype: None,
            data: &vec![],
        };

        let mut parser = RecordParser::new(&rec);
        assert!(result_eq(parser.next_index(), Err(MyError::truncated())), "next_index didn't return truncated error");
    }

    #[test]
    fn test_next_small_index_leaves_valid_state() {
        let rec = Record{
            rectype: None,
            data: &vec![0x03, 0x04],
        };

        let mut parser = RecordParser::new(&rec);
        assert!(result_eq(parser.next_index(), Ok(0x03)), "next_index didn't return proper value");
        assert!(result_eq(parser.next_byte(), Ok(0x04)), "next_index left parser in bad state");
    }

    #[test]
    fn test_next_large_index_succeeds() {
        // Per the spec, if the high bit of the first byte is set,
        // then index = (first_byte & 0x7f) * 0x100 + second_byte
        //
        let rec = Record{
            rectype: None,
            data: &vec![0x81, 0x02],
        };

        let mut parser = RecordParser::new(&rec);
        assert!(result_eq(parser.next_index(), Ok(0x0102)), "next_index didn't return proper value");
    }

    #[test]
    fn test_next_large_index_fails_on_bounds() {
        let rec = Record{
            rectype: None,
            data: &vec![0x80],
        };

        let mut parser = RecordParser::new(&rec);
        assert!(result_eq(parser.next_index(), Err(MyError::truncated())), "next_index didn't return truncated error");
    }

    #[test]
    fn test_next_large_index_leaves_valid_state() {
        let rec = Record{
            rectype: None,
            data: &vec![0x81, 0x02, 0x04],
        };

        let mut parser = RecordParser::new(&rec);
        assert!(result_eq(parser.next_index(), Ok(0x0102)), "next_index didn't return proper value");
        assert!(result_eq(parser.next_byte(), Ok(0x04)), "next_index left parser in bad state");
    }

    #[test]
    fn test_next_rest_str_succeeds() {
        let rec = Record{
            rectype: None,
            data: &vec![0x41, 0x42, 0x43],
        };

        let mut parser = RecordParser::new(&rec);
        assert!(result_eq(parser.rest_str(), Ok("ABC".to_string())), "rest_str didn't return proper value");
    }

    #[test]
    fn test_next_rest_str_succeeds_on_empty_string() {
        let rec = Record{
            rectype: None,
            data: &vec![],
        };

        let mut parser = RecordParser::new(&rec);
        assert!(result_eq(parser.rest_str(), Ok("".to_string())), "rest_str didn't return proper value");
    }

    #[test]
    fn test_next_rest_str_leaves_valid_state() {
        let rec = Record{
            rectype: None,
            data: &vec![0x41, 0x42, 0x43],
        };

        let mut parser = RecordParser::new(&rec);
        assert!(result_eq(parser.rest_str(), Ok("ABC".to_string())), "rest_str didn't return proper value");
        assert!(parser.end(), "rest_str left parser in bad state");

    }

    #[test]
    fn test_next_byte_succeeds() {
        let rec = Record{
            rectype: None,
            data: &vec![0xfe],
        };

        let mut parser = RecordParser::new(&rec);
        assert!(result_eq(parser.next_byte(), Ok(0xfe)), "next_byte didn't return proper value");
    }

    #[test]
    fn test_next_byte_fails_at_end() {
        let rec = Record{
            rectype: None,
            data: &vec![0xfe],
        };

        let mut parser = RecordParser::new(&rec);
        assert!(result_eq(parser.next_byte(), Ok(0xfe)), "next_byte didn't return proper value");
        assert!(result_eq(parser.next_byte(), Err(MyError::truncated())), "next_byte didn't return truncated error");
    }

    #[test]
    fn test_next_byte_leaves_valid_state() {
        let rec = Record{
            rectype: None,
            data: &vec![0xfe, 0x03],
        };

        let mut parser = RecordParser::new(&rec);
        assert!(result_eq(parser.next_byte(), Ok(0xfe)), "first byte not extracted properly" );
        assert!(result_eq(parser.next_byte(), Ok(0x03)), "second byte not extracted properly" );        
    }
}

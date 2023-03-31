use std::io::Read;

use super::parser::LineReader;

pub struct StringReader<'s> {
    string: &'s str,
    cursor: usize,
}

impl<'s> StringReader<'s> {
    pub fn new(string: &'s str) -> Self {
        Self { string, cursor: 0 }
    }
}

impl<'s> Read for StringReader<'s> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        let bytes = self.string.as_bytes();
        let bytes = &bytes[self.cursor..];
        let bytecount = bytes.len().min(buf.len());
        buf[..bytecount].copy_from_slice(&bytes[..bytecount]);
        self.cursor += bytecount;
        Ok(bytecount)
    }
}

impl<'s> LineReader for StringReader<'s> {
    fn next_line(&mut self) -> Result<(String, usize), std::io::Error> {
        let slice = &self.string[self.cursor..];
        for i in 0..slice.len() {
            if slice.as_bytes()[i] == b'\n' {
                self.cursor += i + 1;
                return Ok((slice[..i].to_owned(), i+1));
            }
        }
        self.cursor += slice.len();
        Ok((slice.to_owned(), slice.len()))
    }
}

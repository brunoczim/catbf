//! Utilities to help tracking source code locations and emitting reasonable
//! information in parse error messages.

use std::{fmt, io};

/// Location of an object in the source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Location {
    /// Absolute bytewise position.
    pub position: u64,
    /// Line number.
    pub line: u64,
    /// Column bytewise number.
    pub column: u64,
}

impl Default for Location {
    fn default() -> Self {
        Self::START
    }
}

impl Location {
    /// Location of the beginning of the the source file.
    pub const START: Self = Self { position: 0, line: 1, column: 1 };

    /// Advances the location given the current byte.
    pub fn next(&mut self, byte: u8) {
        self.position += 1;
        if byte == b'\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "line {}, column {}", self.line, self.column)
    }
}

#[derive(Debug)]
pub struct Source<R> {
    bytes: io::Bytes<R>,
    curr_location: Location,
}

impl<R> Source<R>
where
    R: io::Read,
{
    pub fn new(reader: R) -> Self {
        Self { bytes: reader.bytes(), curr_location: Location::START }
    }
}

impl<R> From<R> for Source<R>
where
    R: io::Read,
{
    fn from(reader: R) -> Self {
        Self::new(reader)
    }
}

impl<R> Source<R>
where
    R: io::Read,
{
    pub fn curr_location(&self) -> Location {
        self.curr_location
    }

    pub fn try_next(&mut self) -> io::Result<Option<(u8, Location)>> {
        let Some(byte) = self.bytes.next().transpose()? else {
            return Ok(None);
        };

        let location = self.curr_location();
        self.curr_location.next(byte);
        Ok(Some((byte, location)))
    }
}

impl<R> Iterator for Source<R>
where
    R: io::Read,
{
    type Item = io::Result<(u8, Location)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.try_next().transpose()
    }
}

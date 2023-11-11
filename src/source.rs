use std::{convert::Infallible, fmt, io, iter, slice};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Location {
    pub position: u64,
    pub line: u64,
    pub column: u64,
}

impl Default for Location {
    fn default() -> Self {
        Self::START
    }
}

impl Location {
    pub const START: Self = Self { position: 0, line: 1, column: 1 };

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

pub type IoSource<R> = Source<io::Bytes<R>>;

pub type InfallibleSource<I> =
    Source<iter::Map<I, fn(u8) -> Result<u8, Infallible>>>;

pub type BufSource<'buf> =
    InfallibleSource<iter::Copied<slice::Iter<'buf, u8>>>;

#[derive(Debug, Clone)]
pub struct Source<I> {
    iterator: I,
    curr_location: Location,
}

impl<R> IoSource<R>
where
    R: io::Read,
{
    pub fn from_reader(reader: R) -> Self {
        Self::new(reader.bytes())
    }
}

impl<R> From<R> for IoSource<R>
where
    R: io::Read,
{
    fn from(reader: R) -> Self {
        Self::from_reader(reader)
    }
}

impl<I> InfallibleSource<I>
where
    I: Iterator<Item = u8>,
{
    pub fn from_infallible<T>(iterable: T) -> Self
    where
        T: IntoIterator<IntoIter = I>,
    {
        Self::new(
            iterable.into_iter().map(Ok as fn(u8) -> Result<u8, Infallible>),
        )
    }
}

impl<I> From<I> for InfallibleSource<I>
where
    I: Iterator<Item = u8>,
{
    fn from(iterable: I) -> Self {
        Self::from_infallible(iterable)
    }
}

impl<'buf> BufSource<'buf> {
    pub fn from_buf(buf: &'buf [u8]) -> Self {
        Self::from_infallible(buf.iter().copied())
    }
}

impl<'buf> From<&'buf [u8]> for BufSource<'buf> {
    fn from(buf: &'buf [u8]) -> Self {
        Self::from_buf(buf)
    }
}

impl<I, E> Source<I>
where
    I: Iterator<Item = Result<u8, E>>,
{
    pub fn new<T>(iterable: T) -> Self
    where
        T: IntoIterator<IntoIter = I>,
    {
        Self { iterator: iterable.into_iter(), curr_location: Location::START }
    }

    pub fn curr_location(&self) -> Location {
        self.curr_location
    }

    pub fn try_next(&mut self) -> Result<Option<(u8, Location)>, E> {
        let Some(byte) = self.iterator.next().transpose()? else {
            return Ok(None);
        };

        let location = self.curr_location();
        self.curr_location.next(byte);
        Ok(Some((byte, location)))
    }
}

impl<I> Source<I>
where
    I: Iterator<Item = Result<u8, Infallible>>,
{
    pub fn next_infallible(&mut self) -> Option<(u8, Location)> {
        match self.try_next() {
            Ok(item) => item,
            Err(error) => match error {},
        }
    }
}

impl<I, E> Iterator for Source<I>
where
    I: Iterator<Item = Result<u8, E>>,
{
    type Item = Result<(u8, Location), E>;

    fn next(&mut self) -> Option<Self::Item> {
        self.try_next().transpose()
    }
}

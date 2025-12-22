//! This is a fake IO module for no_std environments

use core::fmt::{Display, Formatter};
use core::result::Result;
use alloc::vec::Vec;


/// This is a fake IO Error type that pretends to be the standard counterpart
#[derive(Debug)]
pub struct Error;

impl Display for Error
{
    fn fmt(&self, _f: &mut Formatter<'_>) -> core::fmt::Result
    {
        // Actually, errors will never occur in this context, so we can just return an unreachable!()
        unreachable!()
    }
}


/// This is a fake IO Write type that pretends to be the standard counterpart
pub trait Write
{
    /// An "infallible" method that pretends to be the standard Write::write() method
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error>;

    /// An "infallible" method that pretends to be the standard Write::write_all() method
    fn write_all(&mut self, buf: &[u8]) -> Result<(), Error>;

    /// An "infallible" method that pretends to be the standard Write::flush() method
    fn flush(&mut self) -> Result<(), Error>;
}


impl Write for Vec<u8>
{
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error>
    {
        self.extend_from_slice(buf);
        Ok(buf.len())
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> Result<(), Error>
    {
        self.extend_from_slice(buf);
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> Result<(), Error>
    {
        Ok(())
    }
}

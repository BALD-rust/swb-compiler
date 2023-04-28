#[cfg(not(feature = "std"))]
use core::error;
#[cfg(feature = "std")]
use std::error;

#[cfg(not(feature = "std"))]
use core::result;
#[cfg(feature = "std")]
use std::result;

#[cfg(not(feature = "std"))]
use core::fmt;
#[cfg(feature = "std")]
use std::fmt;


pub type Result<T> = result::Result<T, Error>;

pub struct Error(pub &'static str);

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl error::Error for Error {

}
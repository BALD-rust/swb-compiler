#[cfg(feature="std")]
use std::fmt;
#[cfg(not(feature="std"))]
use core::fmt;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Address(pub u32);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct AddressRange {
    pub base: Address,
    pub range: u32,
}

impl Address {
    pub fn offset(&self, offset: i32) -> Self {
        Self {
            0: ((self.0 as i32) + offset) as u32
        }
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#06x}", self.0)
    }
}

impl fmt::Display for AddressRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let end = self.base.offset((self.range - 1) as i32);
        write!(f, "{}..{}", self.base, end)
    }
}
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct Address(pub u32);

#[derive(Debug)]
pub struct AddressRange {
    pub base: Address,
    pub range: usize,
}

impl Address {
    pub fn offset(&self, offset: i32) -> Self {
        Self {
            0: ((self.0 as i32) + offset) as u32
        }
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#06x}", self.0)
    }
}

impl Display for AddressRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let end = self.base.offset((self.range - 1) as i32);
        write!(f, "{}..{}", self.base, end)
    }
}
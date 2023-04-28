#[cfg(feature="std")]
use std::fmt;
#[cfg(not(feature="std"))]
use core::fmt;

#[cfg(feature="std")]
use std::convert::TryInto;
#[cfg(not(feature="std"))]
use core::convert::TryInto;

use alloc::vec::Vec;

use crate::address::AddressRange;
use anyhow::{anyhow, Result};
use crate::Address;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum StyleVar {
    Bold = 1,
    Italic = 2,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum Instruction {
    Stop = 0,
    Text(AddressRange) = 1,
    Push(StyleVar) = 2,
    Pop(StyleVar) = 3,
    Endl = 4,
}

impl Instruction {
    fn discriminant(&self) -> u8 {
        // SAFETY: Because we are using repr(u8) this enum is a repr(C) tagged union type.
        // We can read the discriminant in the first u8 field of this struct.
        // see https://doc.rust-lang.org/std/mem/fn.discriminant.html#accessing-the-numeric-value-of-the-discriminant
        unsafe { *<*const _>::from(self).cast::<u8>() }
    }
}

/// - Text: lower 32 bits of the argument are the base address, upper 32 bits are the range
/// - Push/Pop: lower 8 bits of the argument are the style var, upper 56 bits are zero.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct BinaryInstruction {
    pub ty: u8,
    pub arg: u64,
}

impl BinaryInstruction {
    pub fn into_bytes(self) -> [u8; 9] {
        let mut result: [u8; 9] = [self.ty, 0, 0, 0, 0, 0, 0, 0, 0];
        result[1..].copy_from_slice(&self.arg.to_le_bytes());
        result
    }
}

impl TryFrom<[u8; 9]> for BinaryInstruction {
    type Error = anyhow::Error;

    fn try_from(value: [u8; 9]) -> Result<Self> {
        let ty = value[0];
        let arg = u64::from_le_bytes(value[1..].try_into()?);
        Ok(Self {
            ty,
            arg,
        })
    }
}

pub trait ToBinary {
    type Output;
    fn to_binary(&self) -> Self::Output;
}

impl ToBinary for StyleVar {
    type Output = u8;

    fn to_binary(&self) -> Self::Output {
        *self as Self::Output
    }
}

fn pack_u32_into_u64(lower: u32, upper: u32) -> u64 {
    lower as u64 | ((upper as u64) << 32)
}

fn unpack_u64(value: u64) -> (u32, u32) {
    let higher = (value >> 32) as u32;
    let lower = (value & 0xffffffff) as u32;
    (higher, lower)
}

fn parse_address_range(value: u64) -> AddressRange {
    let (higher, lower) = unpack_u64(value);
    AddressRange {
        base: Address(lower),
        range: higher,
    }
}

fn parse_style_var(value: u64) -> Result<StyleVar> {
    match value {
        1 => Ok(StyleVar::Bold),
        2 => Ok(StyleVar::Italic),
        _ => Err(anyhow!("Invalid style var encoding {value}"))
    }
}

impl TryFrom<BinaryInstruction> for Instruction {
    type Error = anyhow::Error;

    fn try_from(value: BinaryInstruction) -> Result<Self> {
        let instruction = match value.ty {
            0 => Ok(Instruction::Stop),
            1 => Ok(Instruction::Text(parse_address_range(value.arg))),
            2 => Ok(Instruction::Push(parse_style_var(value.arg)?)),
            3 => Ok(Instruction::Pop(parse_style_var(value.arg)?)),
            4 => Ok(Instruction::Endl),
            _ => Err(anyhow!("Invalid instruction type {}", value.ty)),
        }?;
        Ok(instruction)
    }
}


impl ToBinary for Instruction {
    type Output = BinaryInstruction;

    fn to_binary(&self) -> Self::Output {
        let ty = self.discriminant();
        match self {
            Instruction::Text(AddressRange { base, range }) => {
                let arg = pack_u32_into_u64(base.0, *range);
                BinaryInstruction {
                    ty,
                    arg,
                }
            },
            Instruction::Push(style) => {
                let arg = *style as u8 as u64;
                BinaryInstruction {
                    ty,
                    arg,
                }
            }
            Instruction::Pop(style) => {
                let arg = *style as u8 as u64;
                BinaryInstruction {
                    ty,
                    arg,
                }
            }
            Instruction::Endl => {
                BinaryInstruction {
                    ty,
                    arg: 0
                }
            }
            Instruction::Stop => {
                BinaryInstruction {
                    ty,
                    arg: 0
                }
            }
        }
    }
}


impl fmt::Display for StyleVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StyleVar::Bold => { write!(f, "bold")?; }
            StyleVar::Italic => { write!(f, "italic")?; }
        };
        Ok(())
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Text(range) => {
                write!(f, "text {range}")?;
            }
            Instruction::Push(style) => {
                write!(f, "push {style}")?;
            }
            Instruction::Pop(style) => {
                write!(f, "pop {style}")?;
            }
            Instruction::Stop => {
                write!(f, "stop")?;
            }
            Instruction::Endl => {
                write!(f, "endl")?;
            }
        };

        Ok(())
    }
}

impl fmt::Display for BinaryInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#0x}:{:#16x}", self.ty, self.arg)
    }
}
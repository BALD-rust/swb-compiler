#[cfg(std)]
use std::fmt;
#[cfg(not(std))]
use core::fmt;

use alloc::vec::Vec;

use crate::address::AddressRange;
use anyhow::{anyhow, Result};

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum StyleVar {
    Bold = 1,
    Italic = 2,
}

#[derive(Debug, Clone, Copy)]
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
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct BinaryInstruction {
    pub ty: u8,
    pub arg: u64,
}

#[derive(Debug, Clone)]
pub struct BinaryProgram {
    pub instructions: Vec<BinaryInstruction>,
}

impl BinaryInstruction {
    pub fn into_bytes(self) -> [u8; 9] {
        let mut result: [u8; 9] = [self.ty, 0, 0, 0, 0, 0, 0, 0, 0];
        result[1..].copy_from_slice(&self.arg.to_le_bytes());
        result
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

impl fmt::Display for BinaryProgram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for instruction in &self.instructions {
            write!(f, "{instruction}\n")?;
        }
        Ok(())
    }
}
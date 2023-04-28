#[cfg(not(feature = "std"))]
use core::fmt;
#[cfg(feature = "std")]
use std::fmt;

#[cfg(not(feature = "std"))]
use core::convert::TryInto;
#[cfg(feature = "std")]
use std::convert::TryInto;

use crate::{BinaryInstruction, Instruction, ToBinary};
use alloc::vec::Vec;
use ascii::{AsAsciiStr, AsciiString, IntoAsciiString};

use crate::Result;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Program {
    pub text: AsciiString,
    pub code: Vec<Instruction>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct BinaryProgram {
    pub text: AsciiString,
    pub code: Vec<BinaryInstruction>,
}

impl TryFrom<&[u8]> for Program {
    type Error = crate::Error;

    fn try_from(value: &[u8]) -> Result<Self> {
        let header = &value[0..8];
        let text_len = u64::from_le_bytes(header.try_into().unwrap()) as usize;
        let text_bytes = &value[8..8 + text_len];
        let mut vec = Vec::with_capacity(text_bytes.len());
        vec.extend_from_slice(text_bytes);
        let text = vec.into_ascii_string().unwrap();
        let instruction_bytes = &value[8 + text_len..];
        let code = instruction_bytes
            .chunks_exact(9)
            .map(|bytes| {
                let mut arr: [u8; 9] = [0, 0, 0, 0, 0, 0, 0, 0, 0];
                arr.copy_from_slice(bytes);
                Instruction::try_from(BinaryInstruction::try_from(arr).unwrap()).unwrap()
            })
            .collect::<Vec<_>>();
        Ok(Self { text, code })
    }
}

impl Program {
    pub fn to_binary(self) -> BinaryProgram {
        BinaryProgram {
            text: self.text,
            code: self
                .code
                .into_iter()
                .map(|instruction| instruction.to_binary())
                .collect(),
        }
    }
}

impl BinaryProgram {
    pub fn into_byte_buffer(self) -> Vec<u8> {
        let len = self.text.len() as u64;
        let header_bytes: [u8; 8] = len.to_le_bytes();
        let mut result = header_bytes.to_vec();
        // We know how many more bytes we need, so this saves some allocations.
        result.reserve(
            (len as usize + self.code.len() * std::mem::size_of::<BinaryInstruction>()) as usize,
        );
        // Add string buffer
        result.extend_from_slice(self.text.as_bytes());
        // Now add our binary instructions
        for instr in self.code {
            let bytes = instr.into_bytes();
            result.extend_from_slice(&bytes);
        }
        result
    }
}

impl fmt::Display for BinaryProgram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for instruction in &self.code {
            write!(f, "{instruction}\n")?;
        }
        Ok(())
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const BLOCK_SIZE: usize = 16;
        write!(f, ".data\n")?;
        // Display our text buffer, we do this in blocks of at most 16 characters
        let mut cur = 0;
        while cur < self.text.len() {
            // Grab at most 16 characters, but never more than the amount of remaining characters
            let remaining = (self.text.len() - cur).min(BLOCK_SIZE);
            if remaining == 0 {
                break;
            }
            let block = self.text.as_slice().get(cur..(cur + remaining)).unwrap();
            let block = unsafe {
                // SAFETY: This slice was created from an AsciiString, so we know there are no non-ascii characters.
                block.as_ascii_str_unchecked()
            };
            write!(f, "\t{:#06x}\t{}\n", cur, block)?;
            cur += remaining;
        }

        write!(f, ".text\n")?;
        for instruction in &self.code {
            write!(f, "\t{instruction}\n")?;
        }

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::*;
    use ascii::AsciiString;

    #[test]
    fn test_from_binary() {
        let program = Program {
            text: AsciiString::from_ascii(*b"Hello").unwrap(),
            code: vec![
                Instruction::Push(StyleVar::Bold),
                Instruction::Text(AddressRange {
                    base: Address(0),
                    range: 5,
                }),
                Instruction::Pop(StyleVar::Bold),
                Instruction::Stop,
            ],
        };
        let bytes = program.clone().to_binary().into_byte_buffer();
        let converted = Program::try_from(bytes.as_slice());
        assert!(converted.is_ok());
        assert_eq!(program, converted.unwrap());
    }
}

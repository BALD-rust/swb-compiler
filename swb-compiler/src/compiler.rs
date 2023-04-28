use anyhow::{anyhow, Error, Result};
use ascii::{AsAsciiStr, AsciiString, FromAsciiError, IntoAsciiString};
use flat_html::{Element, TagKind};
use std::fmt;
use std::fmt::{Display, Formatter};
use swb_shared::{
    Address, AddressRange, BinaryInstruction, BinaryProgram, Instruction, Program, StyleVar,
    ToBinary,
};

fn stylevar_from_tag(tag: TagKind) -> Result<StyleVar> {
    match tag {
        TagKind::Bold => Ok(StyleVar::Bold),
        TagKind::Italic => Ok(StyleVar::Italic),
        _ => Err(anyhow!("could not parse tag into stylevar")),
    }
}

#[derive(Debug)]
pub struct CompilationOutput(pub Program);

#[derive(Debug)]
pub struct CompiledBinary(pub BinaryProgram);

impl CompiledBinary {
    /// Returns a byte buffer with the resulting binary
    /// The first 8 bytes of the buffer contain the length of the data section in bytes.
    pub fn into_byte_buffer(mut self) -> Vec<u8> {
        self.0.into_byte_buffer()
    }
}

impl CompilationOutput {
    pub fn binary(self) -> CompiledBinary {
        CompiledBinary(self.0.to_binary())
    }
}

/// Compiles a flat, possibly reduced HTML representation to SWB
pub fn compile(input: &flat_html::FlatHtml) -> Result<CompilationOutput> {
    let mut output = CompilationOutput(Program {
        text: AsciiString::new(),
        code: vec![],
    });

    output.0.code = input
        .0
        .iter()
        .flat_map(|element| {
            match element {
                Element::Tag(TagKind::LineBreak) => Some(Instruction::Endl),
                Element::Text(data) => {
                    let start = output.0.text.len();
                    let ascii: String = data.chars().filter(|c| c.is_ascii()).collect();
                    output.0.text += &ascii.into_ascii_string().unwrap();
                    Some(Instruction::Text(AddressRange {
                        base: Address(start as u32),
                        range: data.len() as u32,
                    }))
                }
                Element::Tag(kind) => {
                    // Try to parse this tag into a style var, and if so add a push instruction
                    match stylevar_from_tag(kind.clone()) {
                        Ok(value) => Some(Instruction::Push(value)),
                        Err(_) => None,
                    }
                }
                Element::EndTag(kind) => match stylevar_from_tag(kind.clone()) {
                    Ok(value) => Some(Instruction::Pop(value)),
                    Err(_) => None,
                },
                Element::LineBreak => Some(Instruction::Endl),
                Element::IgnoreTag => None,
            }
        })
        .collect();
    output.0.code.push(Instruction::Stop);

    Ok(output)
}

impl Display for CompilationOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        const BLOCK_SIZE: usize = 16;
        write!(f, ".data\n")?;
        // Display our text buffer, we do this in blocks of at most 16 characters
        let mut cur = 0;
        while cur < self.0.text.len() {
            // Grab at most 16 characters, but never more than the amount of remaining characters
            let remaining = (self.0.text.len() - cur).min(BLOCK_SIZE);
            if remaining == 0 {
                break;
            }
            let block = self.0.text.as_slice().get(cur..(cur + remaining)).unwrap();
            let block = unsafe {
                // SAFETY: This slice was created from an AsciiString, so we know there are no ascii characters.
                block.as_ascii_str_unchecked()
            };
            write!(f, "\t{:#06x}\t{}\n", cur, block)?;
            cur += remaining;
        }

        write!(f, ".text\n")?;
        for instruction in &self.0.code {
            write!(f, "\t{instruction}\n")?;
        }

        Ok(())
    }
}

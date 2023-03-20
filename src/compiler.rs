use std::fmt;
use std::fmt::{Display, Formatter};
use anyhow::{anyhow, Error, Result};
use flat_html::{Element, TagKind};
use crate::address::{Address, AddressRange};
use crate::instruction::{Instruction, StyleVar};

#[derive(Debug)]
pub struct CompilationOutput {
    pub text_buf: String,
    pub instructions: Vec<Instruction>
}

/// Compiles a flat, possibly reduced HTML representation to SWB
pub fn compile(input: &flat_html::FlatHtml) -> Result<CompilationOutput> {
    let mut output = CompilationOutput {
        text_buf: String::new(),
        instructions: vec![],
    };

    output.instructions = input.0.iter().flat_map(|element| {
        match element {
            Element::Tag(TagKind::LineBreak) => {
                Some(Instruction::Endl)
            }
            Element::Text(data) => {
                let start = output.text_buf.len();
                output.text_buf += data;
                Some(
                    Instruction::Text(
                        AddressRange {
                            base: Address(start as u32),
                            range: data.len()
                        }
                    )
                )
            }
            Element::Tag(kind) => {
                // Try to parse this tag into a style var, and if so add a push instruction
                match StyleVar::try_from(kind.clone()) {
                    Ok(value) => { Some(Instruction::Push(value)) }
                    Err(_) => { None }
                }
            }
            Element::EndTag(kind) => {
                match StyleVar::try_from(kind.clone()) {
                    Ok(value) => { Some(Instruction::Pop(value)) }
                    Err(_) => { None }
                }
            }
            Element::LineBreak => { Some(Instruction::Endl) }
            Element::IgnoreTag => { None }
        }
    }).collect();
    output.instructions.push(Instruction::Stop);

    Ok(output)
}

impl Display for CompilationOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        const BLOCK_SIZE: usize = 16;
        write!(f, ".data\n")?;
        // Display our text buffer, we do this in blocks of at most 16 characters
        let mut cur = 0;
        while cur < self.text_buf.len() {
            // Grab at most 16 characters, but never more than the amount of remaining characters
            let remaining = (self.text_buf.len() - cur).min(BLOCK_SIZE);
            let block = self.text_buf.get(cur..(cur + remaining)).unwrap();
            write!(f, "\t{:#06x}\t{}\n", cur, block)?;
            cur += remaining;
        }

        write!(f, ".text\n")?;
        for instruction in &self.instructions {
            write!(f, "\t{instruction}\n")?;
        }

        Ok(())
    }
}
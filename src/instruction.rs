use std::fmt::{Display, Formatter};
use flat_html::TagKind;
use crate::address::AddressRange;

use anyhow::{anyhow, Result};

#[derive(Debug)]
pub enum StyleVar {
    Bold,
    Italic,
}

#[derive(Debug)]
pub enum Instruction {
    Text(AddressRange),
    Push(StyleVar),
    Pop(StyleVar),
    Endl,
    Stop,
}

impl TryFrom<TagKind> for StyleVar {
    type Error = anyhow::Error;

    fn try_from(value: TagKind) -> Result<Self> {
        match value {
            TagKind::Bold => { Ok(StyleVar::Bold) }
            TagKind::Italic => { Ok(StyleVar::Italic) }
            _ => { Err(anyhow!("could not parse tag into stylevar")) }
        }
    }
}

impl Display for StyleVar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StyleVar::Bold => { write!(f, "bold")?; }
            StyleVar::Italic => { write!(f, "italic")?; }
        };
        Ok(())
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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

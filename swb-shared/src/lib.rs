#![cfg_attr(not(feature = "std"), no_std)]

#![feature(error_in_core)]

extern crate alloc;
extern crate core;

pub mod instruction;
pub mod address;
pub mod program;
pub mod error;

pub use instruction::*;
pub use address::*;
pub use program::*;
pub use error::*;

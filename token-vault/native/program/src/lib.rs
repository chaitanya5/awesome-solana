use solana_program::entrypoint;

pub mod processor;
pub mod instructions;
pub mod error;
pub mod state;

use crate::processor::process_instruction;

entrypoint!(process_instruction);

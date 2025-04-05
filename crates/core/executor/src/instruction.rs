use core::fmt::Debug;
use serde::{Deserialize, Serialize};

use crate::opcode::Opcode;

/// Instruction.
#[derive(Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct Instruction {
    /// The operation to execute.
    pub opcode: Opcode,
    /// The operand.
    pub operand: u32,
}

impl Instruction {
    /// Create a new instruction.
    pub const fn new(opcode: Opcode) -> Self {
        Self { opcode, operand: 0 }
    }

    /// Create a new jump instruction.
    pub const fn new_jmp(opcode: Opcode, operand: u32) -> Self {
        Self { opcode, operand }
    }
}

impl Instruction {
    pub fn decode_from(opcode: char, operand: Option<u32>) -> Self {
        match opcode {
            '>' => Self::new(Opcode::MemStepForward),
            '<' => Self::new(Opcode::MemStepBackward),
            '+' => Self::new(Opcode::Add),
            '-' => Self::new(Opcode::Sub),
            '.' => Self::new(Opcode::Output),
            ',' => Self::new(Opcode::Input),
            '[' => Self::new_jmp(Opcode::LoopStart, operand.unwrap()),
            ']' => Self::new_jmp(Opcode::LoopEnd, operand.unwrap()),
            _ => unreachable!(),
        }
    }
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.opcode {
            Opcode::LoopStart => f.write_str(&format!("[{}", self.operand))?,
            Opcode::LoopEnd => f.write_str(&format!("]{}", self.operand))?,
            Opcode::Add => f.write_str("+")?,
            Opcode::Sub => f.write_str("-")?,
            Opcode::MemStepForward => f.write_str(">")?,
            Opcode::MemStepBackward => f.write_str("<")?,
            Opcode::Input => f.write_str(",")?,
            Opcode::Output => f.write_str(".")?,
        }
        Ok(())
    }
}

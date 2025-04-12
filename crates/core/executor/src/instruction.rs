use core::fmt::Debug;
use serde::{Deserialize, Serialize};

use crate::opcode::Opcode;

/// Instruction.
#[derive(Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct Instruction {
    /// The operation to execute.
    pub opcode: Opcode,
    /// The first operand.
    pub op_a: u32,
}

impl Instruction {
    /// Create a new instruction.
    pub const fn new(opcode: Opcode) -> Self {
        Self { opcode, op_a: 0 }
    }

    /// Create a new jump instruction.
    pub const fn new_jmp(opcode: Opcode, op_a: u32) -> Self {
        Self { opcode, op_a }
    }

    /// Returns if the instruction is an ALU instruction.
    #[must_use]
    pub const fn is_alu_instruction(&self) -> bool {
        matches!(self.opcode, Opcode::Add | Opcode::Sub)
    }

    /// Returns if the instruction is a Jump instruction.
    #[must_use]
    pub const fn is_jump_instruction(&self) -> bool {
        matches!(self.opcode, Opcode::LoopStart | Opcode::LoopEnd)
    }

    /// Returns if the instruction is a Memory instruction.
    #[must_use]
    pub const fn is_memory_instruction(&self) -> bool {
        matches!(self.opcode, Opcode::MemStepForward | Opcode::MemStepBackward)
    }

    /// Returns if the instruction is a Memory instruction.
    #[must_use]
    pub const fn is_io_instruction(&self) -> bool {
        matches!(self.opcode, Opcode::Input | Opcode::Output)
    }

    #[must_use]
    pub const fn is_mv_immutable(&self) -> bool {
        self.is_alu_instruction() || self.is_jump_instruction() || matches!(self.opcode, Opcode::Output)
    }

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
            Opcode::LoopStart => f.write_str(&format!("[{}", self.op_a))?,
            Opcode::LoopEnd => f.write_str(&format!("]{}", self.op_a))?,
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

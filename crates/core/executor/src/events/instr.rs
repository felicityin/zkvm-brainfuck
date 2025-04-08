use serde::{Deserialize, Serialize};

use crate::opcode::Opcode;

/// Arithmetic Logic Unit (ALU) Event.
///
/// This object encapsulated the information needed to prove an ALU operation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AluEvent {
    /// The program counter.
    pub pc: u32,
    /// The opcode.
    pub opcode: Opcode,
    /// The output operand.
    pub mv_next: u8,
    /// The input operand.
    pub mv: u8,
}

impl AluEvent {
    /// Create a new [`AluEvent`].
    #[must_use]
    pub fn new(pc: u32, opcode: Opcode, mv_next: u8, mv: u8) -> Self {
        Self {
            pc,
            opcode,
            mv_next,
            mv
        }
    }
}

/// Jump Instruction Event.
///
/// This object encapsulated the information needed to prove a jump operation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(C)]
pub struct JmpEvent {
    /// The program counter.
    pub pc: u32,
    /// The next program counter.
    pub next_pc: u32,
    /// The opcode.
    pub opcode: Opcode,
    /// The first operand value.
    pub mp: u32,
    /// The second operand value.
    pub mv: u8,
}

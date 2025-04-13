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
pub struct JumpEvent {
    /// The program counter.
    pub pc: u32,
    /// The next program counter.
    pub next_pc: u32,
    /// The opcode.
    pub opcode: Opcode,
    /// The first operand value.
    pub dst: u32,
    /// The second operand value.
    pub mv: u8,
}

impl JumpEvent {
    /// Create a new [`JumpEvent`].
    #[must_use]
    pub fn new(pc: u32, next_pc: u32, opcode: Opcode, dst: u32, mv: u8) -> Self {
        Self {
            pc,
            next_pc,
            opcode,
            dst,
            mv
        }
    }
}

/// Memory Instruction Event.
///
/// This object encapsulated the information needed to prove a memory operation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(C)]
pub struct MemInstrEvent {
    /// The clk.
    pub clk: u32,
    /// The program counter.
    pub pc: u32,
    /// The opcode.
    pub opcode: Opcode,
    /// The memory pointer.
    pub mp: u32,
    /// The next memory pointer.
    pub next_mp: u32,
}

impl MemInstrEvent {
    /// Create a new [`MemInstrEvent`].
    #[must_use]
    pub fn new(
        clk: u32,
        pc: u32,
        opcode: Opcode,
        mp: u32,
        next_mp: u32,
    ) -> Self {
        Self { clk, pc, opcode, mp, next_mp }
    }
}

/// I/O Instruction Event.
///
/// This object encapsulated the information needed to prove a I/O operation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(C)]
pub struct IoEvent {
    /// The program counter.
    pub pc: u32,
    /// The opcode.
    pub opcode: Opcode,
    /// The memory pointer.
    pub mp: u32,
    /// The memory value.
    pub mv: u8,
}

impl IoEvent {
    /// Create a new [`MemInstrEvent`].
    #[must_use]
    pub fn new(pc: u32, opcode: Opcode, mp: u32, mv: u8) -> Self {
        Self { pc, opcode, mp, mv }
    }
}

use serde::{Deserialize, Serialize};

use crate::events::MemoryRecordEnum;

/// CPU Event.
///
/// This object encapsulates the information needed to prove a CPU operation. This includes its
/// shard, opcode, operands, and other relevant information.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct CpuEvent {
    /// The clock cycle.
    pub clk: u32,
    /// The program counter.
    pub pc: u32,
    /// The next program counter.
    pub next_pc: u32,
    /// Memory pointer.
    pub mp: u32,
    /// For memory instructions.
    pub next_mp: u32,
    /// The memory value.
    pub mv: u8,
    /// For ALU instructions.
    pub next_mv: u8,
    /// The mv memory record.
    pub src_access: Option<MemoryRecordEnum>,
    /// The next_mv memory record.
    pub dst_access: Option<MemoryRecordEnum>,
}

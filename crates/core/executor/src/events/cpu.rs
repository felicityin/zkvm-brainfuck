use serde::{Deserialize, Serialize};

use crate::opcode::Opcode;
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
    /// Jump dst.
    pub jmp_dst: u32,
    /// Memory pointer.
    pub mp: u32,
    /// The first operand.
    pub mv_next: u8,
    /// The first operand memory record.
    pub dst_access: Option<MemoryRecordEnum>,
    /// The second operand.
    pub mv: u8,
    /// The second operand memory record.
    pub src_access: Option<MemoryRecordEnum>,
}

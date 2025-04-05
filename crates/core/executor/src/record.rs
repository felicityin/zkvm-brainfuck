use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::program::Program;
use crate::events::*;

/// A record of the execution of a program.
///
/// The trace of the execution is represented as a list of "events" that occur every cycle.
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionRecord {
    /// The program.
    pub program: Arc<Program>,
    /// A trace of the CPU events which get emitted during execution.
    pub cpu_events: Vec<CpuEvent>,
    /// A trace of the ADD events.
    pub add_events: Vec<AluEvent>,
    /// A trace of the SUB events.
    pub sub_events: Vec<AluEvent>,
    /// A trace of the jump events.
    pub jump_events: Vec<JumpEvent>,
    /// A trace of the memory events.
    pub memory_events: Vec<MemoryEvent>,
}

/// A memory access record.
#[derive(Debug, Copy, Clone, Default)]
pub struct MemoryAccessRecord {
    /// The memory access of the `src` register.
    pub src: Option<MemoryRecordEnum>,
    /// The memory access of the `dst` register.
    pub dst: Option<MemoryRecordEnum>,
}

impl ExecutionRecord {
    /// Create a new [`ExecutionRecord`].
    #[must_use]
    pub fn new(program: Arc<Program>) -> Self {
        Self { program, ..Default::default() }
    }
}

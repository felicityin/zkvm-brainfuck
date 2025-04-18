use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

use crate::events::MemoryRecord;

/// Holds data describing the current state of a program's execution.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[repr(C)]
pub struct ExecutionState {
    /// The program counter.
    pub pc: u32,

    /// The memory register which instructions operate over.
    pub memory_access: HashMap<u32, MemoryRecord>,

    // Memory pointer
    pub mem_ptr: u32,

    /// The global clock keeps track of how many instructions have been executed.
    pub global_clk: u64,

    /// The clock increments by 2 for each instruction that has been executed.
    pub clk: u32,

    /// A stream of input values (global to the entire program).
    pub input_stream: Vec<u8>,

    /// A ptr to the current position in the input stream.
    pub input_stream_ptr: usize,

    /// A stream of public values from the program (global to entire program).
    pub output_stream: Vec<u8>,
}

impl ExecutionState {
    #[must_use]
    /// Create a new [`ExecutionState`].
    pub fn new(input: Vec<u8>) -> Self {
        Self { input_stream: input, ..Default::default() }
    }
}

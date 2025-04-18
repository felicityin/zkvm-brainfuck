use std::sync::Arc;

use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

use bf_stark::MachineRecord;

use crate::events::*;
use crate::program::Program;

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
    /// A trace of the i/o events.
    pub io_events: Vec<IoEvent>,
    /// A trace of the memory instructions.
    pub memory_instr_events: Vec<MemInstrEvent>,
    /// A trace of the memory events.
    pub cpu_memory_access: Vec<MemoryEvent>,
    /// A trace of the byte lookups that are needed.
    pub byte_lookups: HashMap<ByteLookupEvent, usize>,
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

impl ByteRecord for ExecutionRecord {
    fn add_byte_lookup_event(&mut self, blu_event: ByteLookupEvent) {
        *self.byte_lookups.entry(blu_event).or_insert(0) += 1;
    }

    #[inline]
    fn add_byte_lookup_events_from_maps(
        &mut self,
        new_events: Vec<&HashMap<ByteLookupEvent, usize>>,
    ) {
        for new_blu_map in new_events {
            for (blu_event, count) in new_blu_map.iter() {
                *self.byte_lookups.entry(*blu_event).or_insert(0) += count;
            }
        }
    }
}

impl MachineRecord for ExecutionRecord {
    fn append(&mut self, other: &mut ExecutionRecord) {
        self.cpu_events.append(&mut other.cpu_events);
        self.add_events.append(&mut other.add_events);
        self.sub_events.append(&mut other.sub_events);
        self.jump_events.append(&mut other.jump_events);
        self.io_events.append(&mut other.io_events);
        self.memory_instr_events.append(&mut other.memory_instr_events);

        if self.byte_lookups.is_empty() {
            self.byte_lookups = std::mem::take(&mut other.byte_lookups);
        } else {
            self.add_byte_lookup_events_from_maps(vec![&other.byte_lookups]);
        }

        self.cpu_memory_access.append(&mut other.cpu_memory_access);
    }
}

use serde::{Deserialize, Serialize};

/// Memory Event.
///
/// This object encapsulates the information needed to prove a memory access operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEvent {
    /// The address.
    pub addr: u32,
    /// The initial memory access.
    pub initial_mem_access: MemoryRecord,
    /// The final memory access.
    pub final_mem_access: MemoryRecord,
}

/// Memory Record.
///
/// This object encapsulates the information needed to prove a memory access operation.
#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub struct MemoryRecord {
    /// The timestamp.
    pub timestamp: u32,
    /// The value.
    pub value: u8,
}

/// Memory Record Enum.
///
/// This enum represents the different types of memory records that can be stored in the memory
/// event such as reads and writes.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum MemoryRecordEnum {
    /// Read.
    Read(MemoryReadRecord),
    /// Write.
    Write(MemoryWriteRecord),
}

impl From<MemoryReadRecord> for MemoryRecordEnum {
    fn from(read_record: MemoryReadRecord) -> Self {
        MemoryRecordEnum::Read(read_record)
    }
}

impl From<MemoryWriteRecord> for MemoryRecordEnum {
    fn from(write_record: MemoryWriteRecord) -> Self {
        MemoryRecordEnum::Write(write_record)
    }
}

/// Memory Read Record.
///
/// This object encapsulates the information needed to prove a memory read operation.
#[allow(clippy::manual_non_exhaustive)]
#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub struct MemoryReadRecord {
    /// The value.
    pub value: u8,
    /// The timestamp.
    pub timestamp: u32,
    /// The previous timestamp.
    pub prev_timestamp: u32,
}


/// Memory Write Record.
///
/// This object encapsulates the information needed to prove a memory write operation.
#[allow(clippy::manual_non_exhaustive)]
#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub struct MemoryWriteRecord {
    /// The value.
    pub value: u8,
    /// The timestamp.
    pub timestamp: u32,
    /// The previous value.
    pub prev_value: u8,
    /// The previous timestamp.
    pub prev_timestamp: u32,
}

use std::fmt::Display;

use enum_map::Enum;
use p3_field::Field;
use serde::{Deserialize, Serialize};

/// An opcode (short for "operation code") specifies the operation to be performed by the processor.
#[allow(non_camel_case_types)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord, Enum,
)]
pub enum Opcode {
    /// '[': jumps to the matching ] instruction if the currently indicated memory cell is zero
    LoopStart = 0,
    /// ']': jumps to the matching [ if the currently indicated memory cell is nonzero
    LoopEnd = 1,
    /// '+': increments by one (modulo 256) the value of the currently indicated memory cell
    Add = 2,
    /// '-': decrements by one (modulo 256) the value of the currently indicated memory cell
    Sub = 3,
    /// '>': increments the memory pointer by one
    MemStepForward = 4,
    /// '<': decrements the memory pointer by one
    MemStepBackward = 5,
    /// ',': reads a byte from the user input and stores it in the currently indicated memory cell
    Input = 6,
    /// '.': outputs the value of the currently indicated memory cell
    Output = 7,
}

/// Byte Opcode.
///
/// This represents a basic operation that can be performed on a byte. Usually, these operations
/// are performed via lookup tables on that iterate over the domain of two 8-bit values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum ByteOpcode {
    /// Unsigned 8-bit Range Check.
    U8Range = 0,
    /// Unsigned 16-bit Range Check.
    U16Range = 1,
}

impl Opcode {
    /// Get the mnemonic for the opcode.
    #[must_use]
    pub const fn mnemonic(&self) -> &str {
        match self {
            Opcode::LoopStart => "[",
            Opcode::LoopEnd => "]",
            Opcode::Add => "+",
            Opcode::Sub => "-",
            Opcode::MemStepForward => ">",
            Opcode::MemStepBackward => "<",
            Opcode::Input => ",",
            Opcode::Output => ".",
        }
    }

    /// Convert the opcode to a field element.
    #[must_use]
    pub fn as_field<F: Field>(self) -> F {
        F::from_canonical_u32(self as u32)
    }
}

impl Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.mnemonic())
    }
}

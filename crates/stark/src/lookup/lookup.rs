use core::fmt::{Debug, Display};

use p3_air::VirtualPairCol;
use p3_field::Field;

/// A lookup or a permutation argument.
#[derive(Clone)]
pub struct Lookup<F: Field> {
    /// The values of the lookup.
    pub values: Vec<VirtualPairCol<F>>,
    /// The multiplicity of the lookup.
    pub multiplicity: VirtualPairCol<F>,
    /// The kind of lookup.
    pub kind: LookupKind,
}

/// The type of a lookup argument.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LookupKind {
    /// Lookup with the memory table, such as read and write.
    Memory = 1,

    /// Lookup with the program table, loading an instruction at a given pc address.
    Program = 2,

    /// Lookup with the ALU operations.
    Alu = 3,

    /// Lookup with the Jump operations.
    Jump = 4,

    /// Lookup with the byte lookup table for byte operations.
    Byte = 5,

    /// Requesting a range check for a given value and range.
    Range = 6,

    /// Lookup with the field op table for field operations.
    Field = 7,
}

impl LookupKind {
    /// Returns all kinds of lookups.
    #[must_use]
    pub fn all_kinds() -> Vec<LookupKind> {
        vec![
            LookupKind::Memory,
            LookupKind::Program,
            LookupKind::Alu,
            LookupKind::Jump,
            LookupKind::Byte,
            LookupKind::Range,
            LookupKind::Field,
        ]
    }
}

impl<F: Field> Lookup<F> {
    /// Create a new lookup.
    pub const fn new(
        values: Vec<VirtualPairCol<F>>,
        multiplicity: VirtualPairCol<F>,
        kind: LookupKind,
    ) -> Self {
        Self { values, multiplicity, kind }
    }

    /// The index of the argument in the lookup table.
    pub const fn argument_index(&self) -> usize {
        self.kind as usize
    }
}

impl<F: Field> Debug for Lookup<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Lookup")
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}

impl Display for LookupKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LookupKind::Memory => write!(f, "Memory"),
            LookupKind::Program => write!(f, "Program"),
            LookupKind::Alu => write!(f, "Alu"),
            LookupKind::Jump => write!(f, "Jump"),
            LookupKind::Byte => write!(f, "Byte"),
            LookupKind::Range => write!(f, "Range"),
            LookupKind::Field => write!(f, "Field"),
        }
    }
}

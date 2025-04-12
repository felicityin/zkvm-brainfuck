mod memory;
mod program;
mod u8_air;

pub use memory::*;
pub use program::*;
pub use u8_air::*;

use bf_stark::air::{BaseAirBuilder, BfAirBuilder};

/// A trait which contains methods related to memory lookups in an AIR.

pub trait BfCoreAirBuilder:
    BfAirBuilder + U8AirBuilder + MemoryAirBuilder + ProgramAirBuilder
{
}

impl<AB: BaseAirBuilder> MemoryAirBuilder for AB {}
impl<AB: BaseAirBuilder> ProgramAirBuilder for AB {}
impl<AB: BaseAirBuilder> U8AirBuilder for AB {}
impl<AB: BaseAirBuilder + BfAirBuilder> BfCoreAirBuilder for AB {}

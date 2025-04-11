use std::sync::Arc;

use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::events::*;
use crate::program::Program;
use crate::record::{ExecutionRecord, MemoryAccessRecord};
use crate::state::ExecutionState;
use crate::opcode::Opcode;
use crate::instruction::Instruction;

/// The default increment for the program counter.  Is used for all instructions except
/// for branches and jumps.
pub const DEFAULT_PC_INC: u32 = 1;

/// An executor for the zkVM.
///
/// The executor is responsible for executing a user program and tracing important events which
/// occur during execution (i.e., memory reads, alu operations, etc).
#[derive(Default)]
pub struct Executor {
    /// The program.
    pub program: Arc<Program>,

    /// The state of the execution.
    pub state: ExecutionState,

    /// The current trace of the execution that is being collected.
    pub record: ExecutionRecord,

    /// The memory accesses for the current cycle.
    pub memory_accesses: MemoryAccessRecord,

    /// Memory access events.
    pub memory_events: HashMap<u32, MemoryEvent>,
}

/// Errors that the [`Executor`] can throw.
#[derive(Error, Debug, Serialize, Deserialize)]
pub enum ExecutionError {
    /// An error occurred while executing the program.
    #[error("Execution error: {0}")]
    ExecutionError(String),

    /// An error occurred while reading from memory.
    #[error("Memory read error: {0}")]
    MemoryReadError(String),

    /// An error occurred while writing to memory.
    #[error("Memory write error: {0}")]
    MemoryWriteError(String),
}

impl Executor {
    /// Create a new [`Executor`] from a program and options.
    #[must_use]
    pub fn new(program: Program, input: Vec<u8>) -> Self {
        // Create a shared reference to the program.
        let program = Arc::new(program);

        // Create a default record with the program.
        let record = ExecutionRecord::new(program.clone());

        Self {
            program,
            record,
            state: ExecutionState::new(input),
            ..Default::default()
        }
    }

    /// Executes the program.
    /// This function will return an error if the program execution fails.
    pub fn run(&mut self) -> Result<(), ExecutionError> {
        while !self.execute_cycle()? {}

        for (_, event) in self.memory_events.drain() {
            self.record.memory_access.push(event);
        }

        Ok(())
    }

    /// Executes one cycle of the program, returning whether the program has finished.
    #[inline]
    #[allow(clippy::too_many_lines)]
    fn execute_cycle(&mut self) -> Result<bool, ExecutionError> {
        // Fetch the instruction at the current program counter.
        let instruction = self.fetch();

        // Execute the instruction.
        self.execute_instruction(&instruction)?;

        // Increment the clock.
        self.state.global_clk += 1;

        let done = self.state.pc == self.program.instructions.len() as u32;
        Ok(done)
    }

    /// Fetch the instruction at the current program counter.
    #[inline]
    fn fetch(&self) -> Instruction {
        self.program.fetch(self.state.pc)
    }

    /// Execute the given instruction over the current state of the runtime.
    #[allow(clippy::too_many_lines)]
    fn execute_instruction(&mut self, instruction: &Instruction) -> Result<(), ExecutionError> {
        let mut next_pc = self.state.pc.wrapping_add(1);
        let mut dst: u32 = 0;
        let mut mv_next: u8 = 0;
        let mp: u32;
        let mv: u8;

        // Execute the instruction.
        match instruction.opcode {
            Opcode::MemStepForward | Opcode::MemStepBackward => (mp, mv)
                = self.execute_memory(instruction),
            Opcode::Add | Opcode::Sub => (mv_next, mp, mv) = self.execute_alu(instruction),
            Opcode::LoopStart | Opcode::LoopEnd => {
                (mp, mv, next_pc) = self.execute_jump(instruction);
                dst = next_pc;
            }
            Opcode::Input | Opcode::Output => (mp, mv) = self.execute_io(instruction),
        }

        self.emit_events(
            self.state.clk,
            next_pc,
            instruction,
            dst,
            mv_next,
            mp,
            mv,
            self.memory_accesses,
        );

        // Update the program counter.
        self.state.pc = next_pc;

        // Update the clk to the next cycle.
        self.state.clk += 2;
        Ok(())
    }

    /// Execute a memory instruction.
    fn execute_memory(&mut self, instruction: &Instruction) -> (u32, u8) {
        let mp = match instruction.opcode {
            Opcode::MemStepForward => self.state.mem_ptr.wrapping_add(1),
            Opcode::MemStepBackward => self.state.mem_ptr.wrapping_sub(1),
            _ => unreachable!(),
        };
        let mv = self.rr_cpu(mp);
        self.state.mem_ptr = mp;
        (mp, mv)
    }

    /// Execute an ALU instruction.
    fn execute_alu(&mut self, instruction: &Instruction) -> (u8, u32, u8) {
        let mv = self.rr_cpu(self.state.mem_ptr);
        let mv_next = match instruction.opcode {
            Opcode::Add => mv.wrapping_add(1),
            Opcode::Sub => mv.wrapping_sub(1),
            _ => unreachable!(),
        };
        self.rw_cpu(self.state.mem_ptr, mv_next);
        (mv_next, self.state.mem_ptr, mv)
    }

    /// Execute a jump instruction.
    fn execute_jump(&mut self, instruction: &Instruction) -> (u32, u8, u32) {
        let mv = self.rr_cpu(self.state.mem_ptr);
        let next_pc = match instruction.opcode {
            Opcode::LoopStart => {
                if mv == 0 {
                    instruction.operand
                } else {
                    self.state.pc.wrapping_add(1)
                }
            }
            Opcode::LoopEnd => {
                if mv != 0 {
                    instruction.operand
                } else {
                    self.state.pc.wrapping_add(1)
                }
            }
            _ => unreachable!(),
        };
        (self.state.mem_ptr, mv, next_pc)
    }

    /// Execute an IO instruction.
    fn execute_io(&mut self, instruction: &Instruction) -> (u32, u8) {
        let (mp, mv) = match instruction.opcode {
            Opcode::Input => {
                let input = self.state.input_stream[self.state.input_stream_ptr];
                self.rw_cpu(self.state.mem_ptr, input);
                (self.state.mem_ptr, input)
            }
            Opcode::Output => {
                let output = self.rr_cpu(self.state.mem_ptr);
                self.state.output_stream.push(output as u8);
                (self.state.mem_ptr, output)
            }
            _ => unreachable!(),
        };
        (mp, mv)
    }

    /// Emit events for this cycle.
    #[allow(clippy::too_many_arguments)]
    fn emit_events(
        &mut self,
        clk: u32,
        next_pc: u32,
        instruction: &Instruction,
        jmp_dst: u32,
        mv_next: u8,
        mp: u32,
        mv: u8,
        memory_access: MemoryAccessRecord,
    ) {
        self.record.cpu_events.push(CpuEvent {
            clk,
            pc: self.state.pc,
            next_pc,
            jmp_dst,
            mp,
            mv_next,
            dst_access: memory_access.dst,
            mv,
            src_access: memory_access.src,
        });

        if instruction.opcode == Opcode::Add || instruction.opcode == Opcode::Sub {
            self.record.add_events.push(AluEvent::new(self.state.pc, instruction.opcode, mv_next, mv));
        }

        if instruction.opcode == Opcode::LoopStart || instruction.opcode == Opcode::LoopEnd {
            self.record.jump_events.push(JumpEvent::new(
                self.state.pc,
                next_pc,
                instruction.opcode,
                jmp_dst,
                mv,
            ));
        }
    }

    /// Read the memory register.
    #[inline]
    pub fn rr_cpu(&mut self, addr: u32) -> u8 {
        // Read the address from memory and create a memory read record if in trace mode.
        let record = self.rr_traced(addr, self.state.clk);
        self.memory_accesses.src = Some(record.into());
        record.value
    }

    /// Write to a register.
    pub fn rw_cpu(&mut self, register: u32, value: u8) {
        // Read the address from memory and create a memory read record.
        let record = self.rw_traced(register, value, self.state.clk + 1);
        self.memory_accesses.dst = Some(record.into());
    }

    /// Read a register and create an access record.
    pub fn rr_traced(&mut self, addr: u32, timestamp: u32) -> MemoryReadRecord {
        let record: &mut MemoryRecord =  &mut self.state.memory_access.entry(addr)
            .or_insert(MemoryRecord { value: 0, timestamp: 0 });
        let prev_record = *record;
        record.timestamp = timestamp;

        self.memory_events
            .entry(addr)
            .and_modify(|e| {
                e.final_mem_access = *record;
            })
            .or_insert(MemoryEvent {
                addr,
                initial_mem_access: prev_record,
                final_mem_access: *record,
            });

        // Construct the memory read record.
        MemoryReadRecord {
            value: record.value,
            timestamp: record.timestamp,
            prev_timestamp: prev_record.timestamp,
        }
    }

    /// Write a word to a register and create an access record.
    pub fn rw_traced(&mut self, addr: u32, value: u8, timestamp: u32) -> MemoryWriteRecord {
        let record: &mut MemoryRecord =  &mut self.state.memory_access.entry(addr)
            .or_insert(MemoryRecord { value: 0, timestamp: 0 });
        let prev_record = *record;
        record.value = value;
        record.timestamp = timestamp;

        self.memory_events
            .entry(addr)
            .and_modify(|e| {
                e.final_mem_access = *record;
            })
            .or_insert(MemoryEvent {
                addr,
                initial_mem_access: prev_record,
                final_mem_access: *record,
            });

        // Construct the memory write record.
        MemoryWriteRecord {
            value: record.value,
            timestamp: record.timestamp,
            prev_value: prev_record.value,
            prev_timestamp: prev_record.timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use test_artifacts::{FIBO_BF, HELLO_BF, LOOP_BF, MOVE_BF, PRINTA_BF};

    use crate::executor::Executor;
    use crate::program::Program;

    #[test]
    fn test_add_sub_run() {
        let program = Program::from("++-.").unwrap();
        let mut runtime = Executor::new(program, vec![]);
        runtime.run().unwrap();
        assert_eq!(1, runtime.state.output_stream[0]);
    }

    #[test]
    fn test_mem_run() {
        let program = Program::from(">><").unwrap();
        let mut runtime = Executor::new(program, vec![]);
        runtime.run().unwrap();
        assert_eq!(1, runtime.state.mem_ptr);
    }

    #[test]
    fn test_jmp_run() {
        let program = Program::from("[----]").unwrap();
        let mut runtime = Executor::new(program, vec![1]);
        runtime.run().unwrap();
        assert_eq!(2, runtime.state.global_clk);
    }

    #[test]
    fn test_io_run() {
        let program = Program::from(",.").unwrap();
        let mut runtime = Executor::new(program, vec![1]);
        runtime.run().unwrap();
        assert_eq!(1, runtime.state.output_stream[0]);
    }

    #[test]
    fn test_printa_run() {
        let program = Program::from(PRINTA_BF).unwrap();
        println!("program: {:?}", program);

        let mut runtime = Executor::new(program, vec![]);
        runtime.run().unwrap();

        assert_eq!('A' as u8, runtime.state.output_stream[0]);
    }

    #[test]
    fn test_move_run() {
        let program = Program::from(MOVE_BF).unwrap();
        println!("program: {:?}", program);

        let mut runtime = Executor::new(program, vec![]);
        runtime.run().unwrap();

        assert_eq!(2, runtime.state.output_stream[0]);
        assert_eq!(0, runtime.state.output_stream[1]);
    }

    #[test]
    fn test_loop_run() {
        let program = Program::from(LOOP_BF).unwrap();
        println!("program: {:?}", program);

        let mut runtime = Executor::new(program, vec![]);
        runtime.run().unwrap();

        assert_eq!(9, runtime.state.pc);
        assert_eq!(0 as u8, runtime.state.output_stream[0]);
    }

    #[test]
    fn test_hello_run() {
        let program = Program::from(HELLO_BF).unwrap();
        println!("program: {:?}", program);

        let mut runtime = Executor::new(program, vec![]);
        runtime.run().unwrap();

        assert_eq!('H' as u8, runtime.state.output_stream[0]);
        assert_eq!('e' as u8, runtime.state.output_stream[1]);
        assert_eq!('l' as u8, runtime.state.output_stream[2]);
        assert_eq!('l' as u8, runtime.state.output_stream[3]);
        assert_eq!('o' as u8, runtime.state.output_stream[4]);
    }

    #[test]
    fn test_fibo_run() {
        let program = Program::from(FIBO_BF).unwrap();
        println!("program: {:?}", program);

        let mut runtime = Executor::new(program, vec![17]);
        runtime.run().unwrap();

        assert_eq!(85, runtime.state.output_stream[0]);
    }
}

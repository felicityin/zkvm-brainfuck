use anyhow::Result;
use p3_field::PrimeField32;
use serde::{Deserialize, Serialize};

use bf_stark::air::MachineProgram;

use crate::instruction::Instruction;

/// A program that can be executed by the ZKM.
#[derive(PartialEq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct Program {
    pub instructions: Vec<Instruction>,
}

impl Program {
    #[must_use]
    pub fn new(instructions: Vec<Instruction>) -> Self {
        Self { instructions }
    }

    /// Initialize a Brainfuck Program from an appropriate file
    pub fn from(code: &str) -> Result<Program> {
        // keeps track of loop beginnings while (potentially nested) loops are being compiled
        let mut loop_stack = vec![];
        let mut instructions = Vec::new();
        for c in code.chars() {
            // to allow skipping a loop and jumping back to the loop's beginning, the respective start and end positions
            // are recorded in the program
            if c == '[' {
                // placeholder for position of loop's end, to be filled in once position is known
                instructions.push(Instruction::decode_from(c, Some(0)));
                loop_stack.push(instructions.len() - 1);
            } else if c == ']' {
                // record loop's end in beginning
                let start_pos = loop_stack.pop().unwrap();
                instructions[start_pos].op_a = instructions.len() as u32;
                // record loop's start
                instructions.push(Instruction::decode_from(c, Some((start_pos + 1) as u32)));
            } else if c != ' ' && c != '\n' && c != '\r' {
                instructions.push(Instruction::decode_from(c, None));
            }
        }
        Ok(Self { instructions })
    }

    #[must_use]
    /// Fetch the instruction at the given program counter.
    pub fn fetch(&self, pc: u32) -> Instruction {
        self.instructions[pc as usize]
    }
}

impl<F: PrimeField32> MachineProgram<F> for Program {}

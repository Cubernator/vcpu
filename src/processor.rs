mod logic;

use crate::memory::StorageMut;
use crate::{constants, register_index, Address, Endian, Immediate, Register, RegisterId, Word};
use logic::TickResult;

use byteorder::ByteOrder;

pub const fn jmp_addr_i16(offset: i16) -> Immediate {
    offset * (constants::WORD_BYTES as i16)
}

pub const fn jmp_addr_i32(offset: i32) -> Address {
    offset * (constants::WORD_BYTES as i32)
}

pub fn program_from_words(vec: &[Word]) -> Vec<u8> {
    let mut byte_vec = vec![0; vec.len() * constants::WORD_BYTES as usize];
    Endian::write_u32_into(&vec[..], &mut byte_vec[..]);
    byte_vec
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ExitCode {
    /// HALT instruction was executed (Normal shutdown)
    Halted,
    /// Reason for shutdown unknown          
    Unknown,
    /// External termination signal was sent         
    Terminated,
    /// Attempted integer division by zero      
    DivisionByZero,
    /// Attempted to access main memory at invalid address  
    BadMemoryAccess,
    /// Jump address was not aligned to word boundaries
    BadAlignment,
    /// Jump address was out of instruction memory range  
    BadJump,
    /// Opcode or funct was not recognized
    InvalidOpcode,
    /// Program counter is out of instruction memory range
    BadProgramCounter,
}

pub struct Processor {
    registers: [Register; constants::REGISTER_COUNT],
    program_counter: u32,
    state: Option<ExitCode>,
}

impl Processor {
    pub fn new() -> Processor {
        Default::default()
    }

    pub fn register(&self, id: RegisterId) -> &Register {
        &self.registers[register_index(id)]
    }

    pub fn state(&self) -> Option<ExitCode> {
        self.state
    }

    pub fn is_stopped(&self) -> bool {
        self.state.is_some()
    }

    pub fn tick(&mut self, instructions: &[u8], storage: &mut dyn StorageMut) -> Option<ExitCode> {
        if !self.is_stopped() {
            self.state = self.get_new_state(instructions, storage);
        }

        self.state
    }

    fn get_new_state(
        &mut self,
        instructions: &[u8],
        storage: &mut dyn StorageMut,
    ) -> Option<ExitCode> {
        let instr_len = instructions.len() as u32;
        if self.program_counter + constants::WORD_BYTES > instr_len {
            Some(ExitCode::BadProgramCounter)
        } else {
            let pc = self.program_counter as usize;

            let instruction =
                Endian::read_u32(&instructions[pc..(pc + constants::WORD_BYTES as usize)]);

            let tick_result = logic::tick(
                &mut self.registers,
                storage,
                instruction,
                self.program_counter,
            );

            match tick_result {
                TickResult::Next => {
                    let new_pc = self.program_counter.wrapping_add(constants::WORD_BYTES);
                    self.program_counter = if new_pc < instr_len { new_pc } else { 0 };
                    None
                }
                TickResult::Jump(new_pc) => {
                    if (new_pc % (constants::WORD_BYTES as u32)) != 0 {
                        Some(ExitCode::BadAlignment)
                    } else if new_pc >= instr_len {
                        Some(ExitCode::BadJump)
                    } else {
                        self.program_counter = new_pc;
                        None
                    }
                }
                TickResult::Stop(exit_code) => Some(exit_code),
            }
        }
    }

    pub fn run(&mut self, instructions: &[u8], storage: &mut dyn StorageMut) -> ExitCode {
        loop {
            if let Some(exit_code) = self.tick(instructions, storage) {
                return exit_code;
            }
        }
    }
}

impl Default for Processor {
    fn default() -> Processor {
        Processor {
            registers: [Default::default(); constants::REGISTER_COUNT],
            program_counter: 0u32,
            state: None,
        }
    }
}

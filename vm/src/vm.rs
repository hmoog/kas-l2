
use crate::errors::VMResult;
use crate::executable::Executable;
use crate::{Program, Prover};
use solana_sbpf::program::BuiltinProgram;
use solana_sbpf::vm::{ContextObject};
use std::fs;
use std::sync::Arc;

pub struct VM<C: ContextObject> {
    pub builtin_program: Arc<BuiltinProgram<C>>,
}

impl<C: ContextObject> VM<C> {
    pub fn new(builtin_program: Arc<BuiltinProgram<C>>) -> Self {
        Self { builtin_program }
    }

    pub fn load_program(&self, id: [u8; 32], bytes: Vec<u8>) -> VMResult<Program<C>> {
        let mut bytes = &bytes[..];

        Ok(Program {
            executable: Executable::new(
                id,
                Self::read_binary_segment(&mut bytes),
                &self.builtin_program,
            )?,
            prover: Prover::new(id, Self::read_binary_segment(&mut bytes)),
        })
    }

    pub fn load_program_file(&self, id: [u8; 32], path: &str) -> VMResult<Program<C>> {
        self.load_program(id, fs::read(path)?)
    }

    fn read_binary_segment<'a>(cursor: &mut &'a [u8]) -> &'a [u8] {
        if cursor.len() < 8 {
            panic!("Truncated length header");
        }

        let (len_bytes, rest) = cursor.split_at(8);
        let len = u64::from_le_bytes(len_bytes.try_into().unwrap()) as usize;

        if rest.len() < len {
            panic!("Truncated segment: expected {}, have {}", len, rest.len());
        }

        let (segment, rest) = rest.split_at(len);
        *cursor = rest;
        segment
    }
}

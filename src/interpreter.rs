//! Basic Brainfuck interpreter.

use crate::ir::{Instruction, Program};
use std::{io, iter};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ControlError {
    #[error("label {} is out of bounds", .0)]
    BadLabel(usize),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{}", .0)]
    Control(#[from] ControlError),
    #[error("{}", .0)]
    Io(#[from] io::Error),
}

/// A tape allocated for the interpreter.
#[derive(Debug, Clone)]
pub struct Tape {
    cells: Vec<u8>,
    cursor: usize,
}

impl Tape {
    const CHUNK_SIZE: usize = 8192;

    pub fn new() -> Self {
        Self { cells: vec![0; Self::CHUNK_SIZE], cursor: 0 }
    }

    fn inc(&mut self) {
        self.cells[self.cursor] = self.cells[self.cursor].wrapping_add(1);
    }

    fn dec(&mut self) {
        self.cells[self.cursor] = self.cells[self.cursor].wrapping_sub(1);
    }

    fn next(&mut self) {
        self.cursor += 1;
        if self.cursor >= self.cells.len() {
            self.grow_next();
        }
    }

    fn prev(&mut self) {
        if self.cursor == 0 {
            self.grow_prev();
        }
        self.cursor -= 1;
    }

    /// Grows the tape by a chunk (currently 8k) forwards.
    fn grow_next(&mut self) {
        let new_len = self.cells.len() + Self::CHUNK_SIZE;
        self.cells.resize(new_len, 0);
    }

    /// Grows the tape by a chunk (currently 8k) backwards.
    fn grow_prev(&mut self) {
        self.cells.splice(.. 0, iter::repeat(0).take(Self::CHUNK_SIZE));
        self.cursor += Self::CHUNK_SIZE;
    }

    fn input(&mut self, result: Option<u8>) {
        match result {
            Some(byte) => {
                self.cells[self.cursor] = 1;
                self.next();
                self.cells[self.cursor] = byte;
                self.prev();
            },
            None => self.cells[self.cursor] = 0,
        }
    }

    fn output(&self) -> u8 {
        self.cells[self.cursor]
    }

    fn is_zero(&self) -> bool {
        self.cells[self.cursor] == 0
    }
}

#[derive(Debug, Clone)]
pub struct Interface<I, O> {
    input: I,
    output: O,
}

impl<I, O> Interface<I, O>
where
    I: io::Read,
    O: io::Write,
{
    pub fn new(input: I, output: O) -> Self {
        Self { input, output }
    }

    pub fn get(&mut self) -> io::Result<Option<u8>> {
        let mut buf = [0];
        if let Err(error) = self.input.read_exact(&mut buf) {
            if error.kind() == io::ErrorKind::UnexpectedEof {
                Ok(None)
            } else {
                Err(error)
            }
        } else {
            Ok(Some(buf[0]))
        }
    }

    pub fn put(&mut self, byte: u8) -> io::Result<()> {
        self.output.write_all(&[byte])?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct Control {
    program: Program,
    ip: usize,
}

impl Control {
    pub fn new(program: Program) -> Self {
        Self { program, ip: 0 }
    }

    pub fn fetch(&mut self) -> Result<Instruction, ControlError> {
        let Some(instruction) = self.program.code.get(self.ip).copied() else {
            Err(ControlError::BadLabel(self.ip))?
        };

        self.ip += 1;

        Ok(instruction)
    }

    pub fn jump(&mut self, label: usize) {
        self.ip = label;
    }
}

#[derive(Debug, Clone)]
pub struct Machine<I, O> {
    control: Control,
    tape: Tape,
    interface: Interface<I, O>,
}

impl<I, O> Machine<I, O>
where
    I: io::Read,
    O: io::Write,
{
    pub fn new(
        program: Program,
        tape: Tape,
        interface: Interface<I, O>,
    ) -> Self {
        Self { control: Control::new(program), tape, interface }
    }

    pub fn step(&mut self) -> Result<bool, Error> {
        match self.control.fetch()? {
            Instruction::Halt => return Ok(false),
            Instruction::Inc => self.tape.inc(),
            Instruction::Dec => self.tape.dec(),
            Instruction::Next => self.tape.next(),
            Instruction::Prev => self.tape.prev(),
            Instruction::Get => self.tape.input(self.interface.get()?),
            Instruction::Put => self.interface.put(self.tape.output())?,
            Instruction::Jz(label) => {
                if self.tape.is_zero() {
                    self.control.jump(label);
                }
            },
            Instruction::Jnz(label) => {
                if !self.tape.is_zero() {
                    self.control.jump(label);
                }
            },
        }

        Ok(true)
    }

    pub fn run(mut self) -> Result<(), Error> {
        while self.step()? {}
        Ok(())
    }
}

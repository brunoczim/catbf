use crate::source::{Location, Source};
use std::{error::Error, fmt, io, sync::Arc};

#[derive(Debug, Clone)]
pub enum ParseError<E> {
    InputError(E),
    UnmatchedLoopOpen(Location),
    UnmatchedLoopClose(Location),
}

impl<E> fmt::Display for ParseError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InputError(error) => write!(fmtr, "input error: {}", error),
            Self::UnmatchedLoopOpen(location) => {
                write!(fmtr, "unmatched `[`, written at {}", location)
            },
            Self::UnmatchedLoopClose(location) => {
                write!(fmtr, "unmatched `]`, written at {}", location)
            },
        }
    }
}

impl<E> Error for ParseError<E>
where
    E: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InputError(source) => Some(source),
            _ => None,
        }
    }
}

impl<E> From<E> for ParseError<E> {
    fn from(error: E) -> Self {
        Self::InputError(error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Instruction {
    Halt,
    Inc,
    Dec,
    Next,
    Prev,
    Put,
    Get,
    Jz(usize),
    Jnz(usize),
}

mod opcode {
    pub const HALT: u8 = 0;
    pub const INC: u8 = 1;
    pub const DEC: u8 = 2;
    pub const NEXT: u8 = 3;
    pub const PREV: u8 = 4;
    pub const PUT: u8 = 5;
    pub const GET: u8 = 6;
    pub const JZ: u8 = 7;
    pub const JNZ: u8 = 8;
}

mod len {
    pub const OPCODE: usize = (u8::BITS / 8) as usize;
    pub const LABEL: usize = (u32::BITS / 8) as usize;
    pub const NO_ARGS: usize = OPCODE;
    pub const JMP: usize = OPCODE + LABEL;
}

#[derive(Debug, Clone, Default)]
struct Encoder {
    buffer: Vec<u8>,
}

impl Encoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn write(&mut self, dest_pos: usize, bytes: &[u8]) {
        let min_len = dest_pos + bytes.len();
        if self.buffer.len() < min_len {
            self.buffer.resize(min_len, 0);
        }
        self.buffer[dest_pos .. dest_pos + bytes.len()].copy_from_slice(bytes);
    }

    fn encode_opcode(&mut self, dest_pos: usize, opcode: u8) {
        let buf = opcode.to_le_bytes();
        self.write(dest_pos, &buf);
    }

    fn encode_label(&mut self, dest_pos: usize, label: usize) {
        let buf = (label as u32).to_le_bytes();
        self.write(dest_pos, &buf);
    }

    pub fn encode_instr(&mut self, dest_pos: usize, instruction: Instruction) {
        match instruction {
            Instruction::Halt => self.encode_opcode(dest_pos, opcode::HALT),
            Instruction::Inc => self.encode_opcode(dest_pos, opcode::INC),
            Instruction::Dec => self.encode_opcode(dest_pos, opcode::DEC),
            Instruction::Next => self.encode_opcode(dest_pos, opcode::NEXT),
            Instruction::Prev => self.encode_opcode(dest_pos, opcode::PREV),
            Instruction::Get => self.encode_opcode(dest_pos, opcode::GET),
            Instruction::Put => self.encode_opcode(dest_pos, opcode::PUT),
            Instruction::Jz(label) => {
                self.encode_opcode(dest_pos, opcode::JZ);
                self.encode_label(dest_pos + len::OPCODE, label);
            },
            Instruction::Jnz(label) => {
                self.encode_opcode(dest_pos, opcode::JNZ);
                self.encode_label(dest_pos + len::OPCODE, label);
            },
        }
    }

    pub fn ip(&self) -> usize {
        self.buffer.len()
    }

    pub fn finish(self) -> Arc<[u8]> {
        self.buffer.into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Program {
    code: Arc<[u8]>,
}

impl Program {
    pub fn parse<I, E>(mut source: Source<I>) -> Result<Self, ParseError<E>>
    where
        I: Iterator<Item = Result<u8, E>>,
    {
        let mut encoder = Encoder::new();
        let mut loop_starts = Vec::new();
        while let Some((byte, location)) = source.try_next()? {
            let ip = encoder.ip();
            match byte {
                b'+' => encoder.encode_instr(ip, Instruction::Inc),
                b'-' => encoder.encode_instr(ip, Instruction::Dec),
                b'>' => encoder.encode_instr(ip, Instruction::Next),
                b'<' => encoder.encode_instr(ip, Instruction::Prev),
                b',' => encoder.encode_instr(ip, Instruction::Get),
                b'.' => encoder.encode_instr(ip, Instruction::Put),
                b'[' => {
                    loop_starts.push((ip, location));
                    encoder.encode_instr(ip, Instruction::Jz(0));
                },
                b']' => {
                    let Some((label, _)) = loop_starts.pop() else {
                        Err(ParseError::UnmatchedLoopClose(location))?
                    };
                    let loop_body = label + len::JMP;
                    encoder.encode_instr(ip, Instruction::Jnz(loop_body));
                    let loop_end = encoder.ip();
                    encoder.encode_instr(label, Instruction::Jz(loop_end));
                },
                _ => (),
            }
        }

        if let Some((_, location)) = loop_starts.first() {
            Err(ParseError::UnmatchedLoopOpen(*location))?;
        }

        let ip = encoder.ip();
        encoder.encode_instr(ip, Instruction::Halt);

        Ok(Self { code: encoder.finish() })
    }
}

#[derive(Debug, Clone)]
struct Decoder {
    program: Program,
    ip: usize,
}

impl Decoder {
    pub fn new(program: Program) -> Self {
        Self { program, ip: 0 }
    }

    fn read(&mut self, buf: &mut [u8]) {
        buf.copy_from_slice(&self.program.code[self.ip .. self.ip + buf.len()]);
        self.ip += buf.len();
    }

    fn decode_opcode(&mut self) -> u8 {
        let mut buf = [0; len::OPCODE];
        self.read(&mut buf);
        u8::from_le_bytes(buf)
    }

    fn decode_label(&mut self) -> u32 {
        let mut buf = [0; len::LABEL];
        self.read(&mut buf);
        u32::from_le_bytes(buf)
    }

    pub fn decode_instr(&mut self) -> Instruction {
        match self.decode_opcode() {
            opcode::HALT => Instruction::Halt,
            opcode::INC => Instruction::Inc,
            opcode::DEC => Instruction::Dec,
            opcode::NEXT => Instruction::Next,
            opcode::PREV => Instruction::Prev,
            opcode::GET => Instruction::Get,
            opcode::PUT => Instruction::Put,
            opcode::JZ => Instruction::Jz(self.decode_label() as usize),
            opcode::JNZ => Instruction::Jnz(self.decode_label() as usize),
            _ => unreachable!(),
        }
    }

    pub fn jump(&mut self, new_ip: usize) {
        self.ip = new_ip;
    }
}

#[derive(Debug, Clone)]
pub struct Tape {
    cells: Box<[u8]>,
    cursor: usize,
}

impl Tape {
    pub fn new(size: usize) -> Self {
        Self { cells: vec![0; size].into(), cursor: 0 }
    }

    fn inc(&mut self) {
        self.cells[self.cursor] = self.cells[self.cursor].wrapping_add(1);
    }

    fn dec(&mut self) {
        self.cells[self.cursor] = self.cells[self.cursor].wrapping_sub(1);
    }

    fn next(&mut self) {
        if self.cursor + 1 >= self.cells.len() {
            self.cursor = 0;
        } else {
            self.cursor += 1;
        }
    }

    fn prev(&mut self) {
        if self.cursor == 0 {
            self.cursor = self.cells.len() - 1;
        } else {
            self.cursor -= 1;
        }
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
pub struct Machine<I, O> {
    decoder: Decoder,
    tape: Tape,
    input: I,
    output: O,
}

impl<I, O> Machine<I, O>
where
    I: io::Read,
    O: io::Write,
{
    pub fn new(program: Program, tape: Tape, input: I, output: O) -> Self {
        Self { decoder: Decoder::new(program), tape, input, output }
    }

    pub fn step(&mut self) -> io::Result<bool> {
        match self.decoder.decode_instr() {
            Instruction::Halt => return Ok(false),
            Instruction::Inc => self.tape.inc(),
            Instruction::Dec => self.tape.dec(),
            Instruction::Next => self.tape.next(),
            Instruction::Prev => self.tape.prev(),
            Instruction::Get => {
                let mut buf = [0];
                if let Err(error) = self.input.read_exact(&mut buf) {
                    if error.kind() == io::ErrorKind::UnexpectedEof {
                        self.tape.input(None);
                    } else {
                        Err(error)?;
                    }
                } else {
                    self.tape.input(Some(buf[0]));
                }
            },
            Instruction::Put => {
                let buf = [self.tape.output()];
                self.output.write_all(&buf)?;
            },
            Instruction::Jz(label) => {
                if self.tape.is_zero() {
                    self.decoder.jump(label);
                }
            },
            Instruction::Jnz(label) => {
                if !self.tape.is_zero() {
                    self.decoder.jump(label);
                }
            },
        }

        Ok(true)
    }

    pub fn run(mut self) -> io::Result<()> {
        while self.step()? {}
        Ok(())
    }
}

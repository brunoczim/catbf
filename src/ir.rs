//! Intermediate Representation (IR) of Brainfuck programs.

use crate::source::{Location, Source};
use std::{collections::HashSet, fmt, io};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("IO error during parse: {}", .0)]
    IoError(#[from] io::Error),
    #[error("unmatched `[`, written at {}", .0)]
    UnmatchedLoopOpen(Location),
    #[error("unmatched `]`, written at {}", .0)]
    UnmatchedLoopClose(Location),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Instruction {
    /// Inserted when a Brainfuck program reaches its end.
    Halt,
    /// Inserted when a `+` is found. Increments the cell.
    Inc,
    /// Inserted when a `-` is found. Decrements the cell.
    Dec,
    /// Inserted when a `>` is found. Advances the tape.
    Next,
    /// Inserted when a `<` is found. Retracts the tape.
    Prev,
    /// Inserted when a `,` is found. Gets a byte from the stdin.
    Get,
    /// Inserted when a `.` is found. Puts a byte into the stdout.
    Put,
    /// Jumps to the given absolute instruction index when the current cell is
    /// zero. Equivalent to `[`.
    Jz(usize),
    /// Jumps to the given absolute instruction index when the current cell is
    /// not zero. Equivalent to `]`.
    Jnz(usize),
}

impl fmt::Display for Instruction {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Instruction::Halt => write!(fmtr, "halt"),
            Instruction::Inc => write!(fmtr, "inc"),
            Instruction::Dec => write!(fmtr, "dec"),
            Instruction::Next => write!(fmtr, "next"),
            Instruction::Prev => write!(fmtr, "prev"),
            Instruction::Get => write!(fmtr, "get"),
            Instruction::Put => write!(fmtr, "put"),
            Instruction::Jz(label) => write!(fmtr, "jz label_{}", label),
            Instruction::Jnz(label) => write!(fmtr, "jnz label_{}", label),
        }
    }
}

/// A complete Brainfuck program in the IR format.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Program {
    /// Serial list of instructions.
    pub code: Vec<Instruction>,
}

impl Program {
    /// Parses from the given source code reader, yielding a program in the IR
    /// format.
    pub fn parse<R>(mut source: Source<R>) -> Result<Self, ParseError>
    where
        R: io::Read,
    {
        let mut code = Vec::new();
        let mut loop_starts = Vec::new();

        while let Some((byte, location)) = source.try_next()? {
            match byte {
                b'+' => code.push(Instruction::Inc),
                b'-' => code.push(Instruction::Dec),
                b'>' => code.push(Instruction::Next),
                b'<' => code.push(Instruction::Prev),
                b',' => code.push(Instruction::Get),
                b'.' => code.push(Instruction::Put),
                b'[' => {
                    let ip = code.len();
                    loop_starts.push((ip, location));
                    code.push(Instruction::Jz(0));
                },
                b']' => {
                    let Some((label, _)) = loop_starts.pop() else {
                        Err(ParseError::UnmatchedLoopClose(location))?
                    };
                    let loop_body = label + 1;
                    code.push(Instruction::Jnz(loop_body));
                    let loop_end = code.len();
                    code[label] = Instruction::Jz(loop_end);
                },
                _ => (),
            }
        }

        if let Some((_, location)) = loop_starts.first() {
            Err(ParseError::UnmatchedLoopOpen(*location))?;
        }

        code.push(Instruction::Halt);

        Ok(Self { code })
    }
}

impl fmt::Display for Program {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        let labels: HashSet<usize> = self
            .code
            .iter()
            .filter_map(|instruction| match *instruction {
                Instruction::Jz(label) | Instruction::Jnz(label) => Some(label),
                _ => None,
            })
            .collect();
        for (i, instruction) in self.code.iter().enumerate() {
            if labels.contains(&i) {
                write!(fmtr, "label_{}:\n", i)?;
            }
            write!(fmtr, "    {}\n", instruction)?;
        }
        Ok(())
    }
}

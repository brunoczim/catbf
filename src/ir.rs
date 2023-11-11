use crate::source::{Location, Source};
use std::{collections::HashSet, error::Error, fmt};
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum ParseError<E: Error> {
    #[error("input error: {}", .0)]
    InputError(#[from] E),
    #[error("unmatched `[`, written at {}", .0)]
    UnmatchedLoopOpen(Location),
    #[error("unmatched `]`, written at {}", .0)]
    UnmatchedLoopClose(Location),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Instruction {
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Program {
    pub code: Vec<Instruction>,
}

impl Program {
    pub fn parse<I, E>(mut source: Source<I>) -> Result<Self, ParseError<E>>
    where
        I: Iterator<Item = Result<u8, E>>,
        E: Error,
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

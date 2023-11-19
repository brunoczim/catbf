use crate::ir::{Instruction, Program};
use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
    process::{Command, Stdio},
};
use thiserror::Error;

pub const TARGET_SUPPORTED: bool =
    cfg!(all(target_os = "linux", target_arch = "x86_64"));

#[derive(Debug, Error)]
pub enum Error {
    #[error("target is unsupported for Ahead-Of-Time compilation")]
    UnsupportedTarget,
    #[error("label index {} is out of bounds", .0)]
    BadLabelIndex(usize),
    #[error("{}: {}", .0.display(), .1)]
    Io(PathBuf, io::Error),
}

pub fn compile<P>(program: &Program, directory: P) -> Result<(), Error>
where
    P: Into<PathBuf>,
{
    let mut path = directory.into();

    if !TARGET_SUPPORTED {
        Err(Error::UnsupportedTarget)?;
    }

    fs::create_dir_all(&path)
        .map_err(|error| Error::Io(path.clone(), error))?;

    generate_runtime_source(&mut path)?;

    generate_prog_asm(program, &mut path)?;

    link(&mut path)?;

    Ok(())
}

fn link(path: &mut PathBuf) -> Result<(), Error> {
    let mut command = Command::new("cc");
    path.push("runtime.c");
    command.arg(&path);
    path.pop();
    path.push("prog.s");
    command.arg(&path);
    path.pop();
    command.arg("-o");
    path.push("prog");
    command.arg(&path);
    path.pop();
    command.stdout(Stdio::inherit());
    let mut spawned =
        command.spawn().map_err(|error| Error::Io(path.clone(), error))?;
    spawned.wait().map_err(|error| Error::Io(path.clone(), error))?;
    Ok(())
}

fn generate_runtime_source(path: &mut PathBuf) -> Result<(), Error> {
    path.push("runtime.c");

    fs::write(&path, include_str!("../../resources/x86_64/linux/runtime.c"))
        .map_err(|error| Error::Io(path.clone(), error))?;

    path.pop();

    Ok(())
}

fn generate_prog_asm(
    program: &Program,
    path: &mut PathBuf,
) -> Result<(), Error> {
    path.push("prog.s");

    let mut prog_file = fs::File::create(&path)
        .map_err(|error| Error::Io(path.clone(), error))?;

    prog_file
        .write_all(include_bytes!("../../resources/x86_64/linux/preamble.s"))
        .map_err(|error| Error::Io(path.clone(), error))?;

    prog_file
        .write_all(include_bytes!("../../resources/x86_64/linux/enter.s"))
        .map_err(|error| Error::Io(path.clone(), error))?;

    for (i, instruction) in program.code.iter().copied().enumerate() {
        write!(prog_file, ".label_{}:\n", i)
            .map_err(|error| Error::Io(path.clone(), error))?;

        match instruction {
            Instruction::Halt => {
                prog_file
                    .write_all(include_bytes!(
                        "../../resources/x86_64/linux/halt.s"
                    ))
                    .map_err(|error| Error::Io(path.clone(), error))?;
            },

            Instruction::Inc => {
                prog_file
                    .write_all(include_bytes!(
                        "../../resources/x86_64/linux/inc.s"
                    ))
                    .map_err(|error| Error::Io(path.clone(), error))?;
            },

            Instruction::Dec => {
                prog_file
                    .write_all(include_bytes!(
                        "../../resources/x86_64/linux/dec.s"
                    ))
                    .map_err(|error| Error::Io(path.clone(), error))?;
            },

            Instruction::Next => {
                let label = format!(".growed_next_{}", i);
                let content =
                    include_str!("../../resources/x86_64/linux/next.s")
                        .replace(".growed_next", &label);
                prog_file
                    .write_all(content.as_bytes())
                    .map_err(|error| Error::Io(path.clone(), error))?;
            },

            Instruction::Prev => {
                let label = format!(".growed_prev_{}", i);
                let content =
                    include_str!("../../resources/x86_64/linux/prev.s")
                        .replace(".growed_prev", &label);
                prog_file
                    .write_all(content.as_bytes())
                    .map_err(|error| Error::Io(path.clone(), error))?;
            },

            Instruction::Get => {
                let label = format!(".get_growed_next_{}", i);
                let content =
                    include_str!("../../resources/x86_64/linux/get.s")
                        .replace(".get_growed_next", &label);
                prog_file
                    .write_all(content.as_bytes())
                    .map_err(|error| Error::Io(path.clone(), error))?;
            },

            Instruction::Put => {
                prog_file
                    .write_all(include_bytes!(
                        "../../resources/x86_64/linux/put.s"
                    ))
                    .map_err(|error| Error::Io(path.clone(), error))?;
            },

            Instruction::Jz(to_i) => {
                let label = format!(".label_{}", to_i);
                let content = include_str!("../../resources/x86_64/linux/jz.s")
                    .replace(".jz_label", &label);
                prog_file
                    .write_all(content.as_bytes())
                    .map_err(|error| Error::Io(path.clone(), error))?;
            },

            Instruction::Jnz(to_i) => {
                let label = format!(".label_{}", to_i);
                let content =
                    include_str!("../../resources/x86_64/linux/jnz.s")
                        .replace(".jnz_label", &label);
                prog_file
                    .write_all(content.as_bytes())
                    .map_err(|error| Error::Io(path.clone(), error))?;
            },
        }
    }

    write!(prog_file, ".label_{}:\n", program.code.len())
        .map_err(|error| Error::Io(path.clone(), error))?;

    prog_file
        .write_all(include_bytes!("../../resources/x86_64/linux/leave.s"))
        .map_err(|error| Error::Io(path.clone(), error))?;

    path.pop();

    Ok(())
}

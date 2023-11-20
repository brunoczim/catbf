use catbf::{
    compiler::{aot, jit},
    interpreter::{Interface, Machine, Tape},
    ir::Program,
    source::Source,
};
use clap::Parser;
use std::{
    fs::File,
    io::{self, BufReader},
    path::PathBuf,
    process,
};

/// A complete brainfuck implementation: interpreter, Ahead-Of-Time (AOT)
/// compiler and Just-In-Time (JIT) compiler.
///
/// The tape is "infinite" both forwards and backwards. Cells are 8-bit. Get
/// from stdin writes to two cells: the first is a "boolean" indicating whether
/// a byte was read (false = EOF), the secon second is the byte read.
#[derive(Debug, Clone, Parser)]
struct Cli {
    /// Source file path.
    path: PathBuf,
    /// Print intermediate representation.
    #[arg(short = 'p', long = "print-ir")]
    print_ir: bool,
    /// Compile the program Ahead-Of-Time (AOT) and place the artifacts into
    /// the directory indetified by the given path.
    #[arg(short = 'o', long = "compile-to")]
    compile_aot: Option<PathBuf>,
    /// Compile the program Just-In-Time (JIT) and run it. If the target
    /// platform is not supported, this will fallback to an interpreted
    /// execution.
    #[arg(short = 'j', long = "jit", conflicts_with = "force_jit")]
    jit: bool,
    /// Force Just-In-Time (JIT) compilation of the program and run it. If the
    /// target platform is not supported, this will fail and the program will
    /// not be executed.
    #[arg(short = 'J', long = "force-jit", conflicts_with = "jit")]
    force_jit: bool,
}

fn try_main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let reader = BufReader::new(File::open(cli.path)?);
    let source = Source::from_reader(reader);
    let program = Program::parse(source)?;
    if cli.print_ir {
        println!("{}", program);
    } else if let Some(directory) = cli.compile_aot {
        aot::compile(&program, directory)?;
    } else {
        if cli.force_jit || (cli.jit && jit::TARGET_SUPPORTED) {
            let executable = jit::compile(&program)?;
            executable.run(io::stdin(), io::stdout())?;
        } else {
            let tape = Tape::new();
            let interface = Interface::new(io::stdin(), io::stdout());
            let machine = Machine::new(program, tape, interface);
            machine.run()?;
        }
    }
    Ok(())
}

fn main() {
    if let Err(error) = try_main() {
        eprintln!("{}", error);
        process::exit(1);
    }
}

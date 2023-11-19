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

#[derive(Debug, Clone, Parser)]
struct Cli {
    path: PathBuf,
    #[arg(short = 'r', long = "print-ir")]
    print_ir: bool,
    #[arg(short = 'o', long = "compile-to")]
    compile_aot: Option<PathBuf>,
    #[arg(short = 'j', long = "jit", conflicts_with = "force_jit")]
    jit: bool,
    #[arg(short = 'J', long = "force-jit", conflicts_with = "jit")]
    force_jit: bool,
}

fn try_main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let reader = BufReader::new(File::open(cli.path)?);
    let source = Source::from_reader(reader);
    let program = Program::parse(source)?;
    let mut execute = true;
    if cli.print_ir {
        println!("{}", program);
        execute = false;
    }

    if let Some(directory) = cli.compile_aot {
        aot::compile(&program, directory)?;
        execute = false;
    }

    if execute {
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

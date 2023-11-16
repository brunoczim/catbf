use catbf::{
    compiler::aot,
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
}

fn try_main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let reader = BufReader::new(File::open(cli.path)?);
    let source = Source::from_reader(reader);
    let program = Program::parse(source)?;
    let mut interpret = true;
    if cli.print_ir {
        println!("{}", program);
        interpret = false;
    }

    if let Some(directory) = cli.compile_aot {
        aot::compile(&program, directory)?;
        interpret = false;
    }

    if interpret {
        let tape = Tape::new();
        let interface = Interface::new(io::stdin(), io::stdout());
        let machine = Machine::new(program, tape, interface);
        machine.run()?;
    }
    Ok(())
}

fn main() {
    if let Err(error) = try_main() {
        eprintln!("{}", error);
        process::exit(1);
    }
}

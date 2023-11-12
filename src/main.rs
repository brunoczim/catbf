use clap::Parser;
use fast_bfc::{
    interpreter::{Interface, Machine, Tape},
    ir::Program,
    source::Source,
};
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
}

fn try_main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let reader = BufReader::new(File::open(cli.path)?);
    let source = Source::from_reader(reader);
    let program = Program::parse(source)?;
    if cli.print_ir {
        println!("{}", program);
    } else {
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

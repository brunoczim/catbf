use clap::Parser;
use fast_bfc::{
    interpreted::serialized::{Machine, Program, Tape},
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
    #[arg(short = 's', long = "tape-size", default_value = "65536")]
    tape_size: usize,
}

fn try_main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let reader = BufReader::new(File::open(cli.path)?);
    let source = Source::from_reader(reader);
    let program = Program::parse(source)?;
    let tape = Tape::new(cli.tape_size);
    let machine = Machine::new(program, tape, io::stdin(), io::stdout());
    machine.run()?;
    Ok(())
}

fn main() {
    if let Err(error) = try_main() {
        eprintln!("{}", error);
        process::exit(1);
    }
}

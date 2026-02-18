use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "comparer",
    version = "0.1.0",
    about = "Reads an input file and prints its contents in the specified output format",
    long_about = None,
    arg_required_else_help = true
)]
struct Args {
    /// Input file path
    #[arg(short, long, value_name = "FILE")]
    input: PathBuf,

    /// Input format (csv, txt, bin)
    #[arg(short = 'f', long, value_enum, value_name = "FORMAT")]
    input_format: Format,

    /// Output format (csv, txt, bin)
    #[arg(short = 't', long, value_enum, value_name = "FORMAT")]
    output_format: Format,
}

#[derive(Debug, Clone, ValueEnum)]
enum Format {
    Csv,
    Txt,
    Bin,
}

fn main() {
    let args = Args::parse();
    println!("{:#?}", args);
}

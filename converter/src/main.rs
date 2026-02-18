use anyhow::Context;
use clap::{Parser, ValueEnum};
use parser::{Bin, Csv, Decoder, Encoder, Transaction, Txt};
use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "converter",
    version = "0.1.0",
    about = "Reads an input file and prints its contents in the specified output format",
    long_about = None,
    arg_required_else_help = true
)]
struct Args {
    /// Input file path
    #[arg(short, long, value_name = "FILE")]
    input: PathBuf,

    /// Input format
    #[arg(short = 'f', long, value_name = "FORMAT")]
    input_format: Format,

    /// Output format
    #[arg(short = 't', long, value_name = "FORMAT")]
    output_format: Format,
}

#[derive(Debug, Clone, ValueEnum)]
enum Format {
    Csv,
    Txt,
    Bin,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let file = File::open(&args.input)
        .with_context(|| format!("failed to open file: {}", args.input.display()))?;

    let mut reader = BufReader::new(file);
    let txs = decode(&args.input_format, &mut reader)?;

    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    encode(&args.output_format, &*txs, &mut out)?;
    out.flush()?;
    Ok(())
}

fn decode(format: &Format, reader: &mut impl io::Read) -> anyhow::Result<Vec<Transaction>> {
    match format {
        Format::Csv => Csv.decode(reader),
        Format::Txt => Txt.decode(reader),
        Format::Bin => Bin.decode(reader),
    }
    .context("failed to decode transactions")
}

fn encode(format: &Format, txs: &[Transaction], writer: &mut impl io::Write) -> anyhow::Result<()> {
    match format {
        Format::Csv => Csv.encode(txs, writer),
        Format::Txt => Txt.encode(txs, writer),
        Format::Bin => Bin.encode(txs, writer),
    }
    .context("failed to encode transactions")
}

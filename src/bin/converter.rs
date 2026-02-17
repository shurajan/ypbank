use std::fs::File;
use std::io::{BufReader, BufWriter, Write};

use ypbank::{Bin,Csv,Txt, Decoder, Encoder, };

fn main() {
    // Path to the binary test file
    let path = "tests/data/records_example.bin";

    // Open file
    let file = File::open(path).expect("failed to open bin file");

    // Wrap into buffered reader
    let mut reader = BufReader::new(file);
    let txs = Bin.decode(&mut reader).expect("failed to decode transactions");

    let csv_file = File::create("tests/data/test_output.csv").unwrap();
    let mut csv_writer = BufWriter::new(csv_file);
    Csv.encode(&*txs, &mut csv_writer).expect("failed to encode transactions");
    csv_writer.flush().unwrap();

    let txt_file = File::create("tests/data/test_output.txt").unwrap();
    let mut txt_writer = BufWriter::new(txt_file);
    Txt.encode(&*txs, &mut txt_writer).expect("failed to encode transactions");
    txt_writer.flush().unwrap();

    println!("Decoded {} transactions:\n", txs.len());

    for (i, tx) in txs.iter().enumerate() {
        println!("--- Transaction {} ---", i + 1);
        println!("ID:          {}", tx.tx_id);
        println!("Type:        {}", tx.tx_type);
        println!("From user:   {}", tx.from_user_id);
        println!("To user:     {}", tx.to_user_id);
        println!("Amount:      {}", tx.amount);
        println!("Timestamp:   {}", tx.timestamp);
        println!("Status:      {}", tx.status);
        println!("Description: {}", tx.description);
        println!();
    }
}
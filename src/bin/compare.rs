use std::fs::File;
use std::io::{BufReader};

use ypbank::{Csv, Decoder};

fn main() {

    let path1 = "tests/data/records_example.csv";
    let file1 = File::open(path1).expect("failed to open bin file");
    let mut reader1 = BufReader::new(file1);
    let txs1 = Csv.decode(&mut reader1).expect("failed to decode transactions");

    let path2 = "tests/data/records_example.csv";
    let file2 = File::open(path2).expect("failed to open bin file");
    let mut reader2 = BufReader::new(file2);
    let txs2 = Csv.decode(&mut reader2).expect("failed to decode transactions");


    println!("Decoded {} transactions:\n", txs1.len());
    println!("Decoded {} transactions:\n", txs2.len());
}
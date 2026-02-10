use std::io::Cursor;
use ypbank::{formats, read_transactions};

fn main() {
    let decoder = formats::Csv;
    let mut cursor = Cursor::new(b"id,amount,currency\n1,10,USD\n");

    let txs = read_transactions(&decoder, &mut cursor).unwrap();
    println!("{:?}", txs);
}

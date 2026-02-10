use std::io::Cursor;

fn main() {
   println!("Hello, world!");

    let cursor = Cursor::new(b"id,amount,currency\n1,10,USD\n");
    let txs = read_transactions(&Csv, cursor)?;
}

use std::io::Cursor;
use ypbank::csv::Csv;
use ypbank::read_transactions;

fn main() {
    let decoder = Csv;
    let data = br#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1001,DEPOSIT,0,501,50000,1672531200000,FAILURE,"Initial account funding"
"#;

    let mut cursor = Cursor::new(data);

    let txs = read_transactions(&decoder, &mut cursor).unwrap();
    println!("{:?}", txs);
}

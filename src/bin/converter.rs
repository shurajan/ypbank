use std::io::Cursor;
use ypbank::{Decoder, Txt};

fn main() {
    let data = br#"# Record 1 (Deposit)
TX_ID: 1234567890123456
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 9876543210987654
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Terminal deposit"

# Record 2 (Transfer)
TX_ID: 2312321321321321
TIMESTAMP: 1633056800000
STATUS: FAILURE
TX_TYPE: TRANSFER
FROM_USER_ID: 1231231231231231
TO_USER_ID: 9876543210987654
AMOUNT: 1000
DESCRIPTION: "User transfer"

# Record 3 (Withdrawal)
TX_ID: 3213213213213213
AMOUNT: 100
TX_TYPE: WITHDRAWAL
FROM_USER_ID: 9876543210987654
TO_USER_ID: 0
TIMESTAMP: 1633066800000
STATUS: SUCCESS
DESCRIPTION: "User withdrawal"
"#;

    let mut cursor = Cursor::new(data);

    match Txt.decode(&mut cursor) {
        Ok(txs) => {
            println!("Parsed {} transactions:", txs.len());
            for tx in &txs {
                println!("  {:?}", tx);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}

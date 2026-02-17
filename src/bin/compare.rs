fn main() {
    use std::io::Cursor;
    use ypbank::{Csv, Encoder, Transaction, TxStatus, TxType};

    let txs = vec![Transaction {
        tx_id: 1001,
        tx_type: TxType::Deposit,
        from_user_id: 0,
        to_user_id: 501,
        amount: 50000,
        timestamp: 1672531200000,
        status: TxStatus::Success,
        description: "\"Test\"".to_string(),
    }];

    let mut buffer = Cursor::new(Vec::new());
    let result = Csv.encode(&txs, &mut buffer);
    assert!(result.is_ok());
}

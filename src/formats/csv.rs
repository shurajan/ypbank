use crate::error::{ReaderError, WriterError};
use crate::transaction::{Transaction, TransactionDecoder, TransactionEncoder, TxStatus, TxType};
use crate::{formats, read_transactions};
use std::io::{BufRead, BufReader, Cursor, Read, Write};

pub struct Csv;

impl TransactionDecoder for Csv {
    fn decode_all<R: Read>(&self, r: &mut R) -> Result<Vec<Transaction>, ReaderError> {
        let mut txs = Vec::new();

        let mut reader = BufReader::new(r); // BufReader<&mut R>

        //Проверяем заголовок
        let mut header = String::new();
        reader.read_line(&mut header).map_err(ReaderError::Io)?;

        let cols: Vec<&str> = header.trim().split(',').collect();

        // 3) проверяем структуру
        let expected = [
            "TX_ID",
            "TX_TYPE",
            "FROM_USER_ID",
            "TO_USER_ID",
            "AMOUNT",
            "TIMESTAMP",
            "STATUS",
            "DESCRIPTION",
        ];

        if cols != expected {
            return Err(ReaderError::InvalidFormat);
        }

        for (i, line) in reader.lines().enumerate() {
            let line = line.map_err(ReaderError::Io)?;

            // Пропускаем пустые строки
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.trim().split(',').collect();

            if parts.len() != 8 {
                return Err(ReaderError::InvalidDataFormat {
                    field_name: line.to_string(),
                    line: i + 2,
                });
            }

            let tx_id: u64 = parts[0].parse().map_err(|_| {
                ReaderError::InvalidDataFormat {
                    field_name: "TX_ID".to_string(),
                    line: i + 2,
                }
            })?;

            txs.push(Transaction { tx_id, tx_type: TxType::Deposit, from_user_id: 0, to_user_id: 0, amount: 0, timestamp: 0, status: TxStatus::Success, description: "".to_string() });
        }

        Ok(txs)
    }
}

impl TransactionEncoder for Csv {
    fn encode_all<W: Write>(&self, txs: &Vec<Transaction>, w: &mut W) -> Result<(), WriterError> {
        todo!()
    }
}

#[test]
fn test_wrong_header() {
    let decoder = formats::Csv;

    // Заголовок неправильный (не совпадает с expected)
    let data = br#"wrong,header,columns
1,2,3
"#;

    let mut cursor = Cursor::new(data);

    // unwrap_err() возвращает ошибку
    let err = read_transactions(&decoder, &mut cursor).unwrap_err();

    // Проверяем, что это именно InvalidFormat
    assert!(matches!(err, ReaderError::InvalidFormat));
}

#[test]
fn test_correct_header() {
    let decoder = formats::Csv;

    let data = br#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,"Initial account funding"
"#;

    let mut cursor = Cursor::new(data);

    // Должно быть Ok, потому что header правильный
    let result = read_transactions(&decoder, &mut cursor);

    assert!(result.is_ok());
}


#[test]
fn test_correct_data() {
    let decoder = formats::Csv;

    let data = br#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,"Initial account funding"
1002,TRANSFER,501,502,15000,1672534800000,FAILURE,"Payment for services, invoice #123"
1003,WITHDRAWAL,502,0,1000,1672538400000,PENDING,"ATM withdrawal"
"#;

    let mut cursor = Cursor::new(data);

    // Должно быть Ok, потому что header правильный
    let result = read_transactions(&decoder, &mut cursor);

    assert_eq!(result.unwrap().len(), 3);
}
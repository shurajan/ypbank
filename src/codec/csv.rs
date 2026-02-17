use crate::error::{ReaderError, WriterError};
use crate::transaction::{Transaction, schema};
use crate::{Decoder, Encoder};
use std::io::{BufRead, BufReader, Read, Write};

// ─────────────────────────────────────────────────────────────────────────────
// Public types
// ─────────────────────────────────────────────────────────────────────────────
/// CSV transaction codec.
///
/// This codec implements [`Decoder`] and [`Encoder`] for the CSV format.
///
/// The expected input format contains a header row:
///
/// ```text
/// TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
/// ```
///
/// ## Example: Decode a single transaction
///
/// ```
/// use ypbank::{Decoder, Csv};
///
/// let data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
/// 1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,"Initial funding"
/// "#;
///
/// let txs = Csv.decode(&mut data.as_bytes()).unwrap();
///
/// assert_eq!(txs.len(), 1);
/// assert_eq!(txs[0].tx_id, 1001);
/// ```
///
/// ## Example: Decode multiple transactions
///
/// ```
/// use ypbank::{Decoder, Csv};
///
/// let data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
/// 1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,"First"
/// 1002,TRANSFER,501,502,15000,1672534800000,FAILURE,"Second"
/// "#;
///
/// let txs = Csv.decode(&mut data.as_bytes()).unwrap();
///
/// assert_eq!(txs.len(), 2);
/// ```
///
/// ## Errors
///
/// Returns [`ReaderError`] if:
///
/// - the CSV is malformed
/// - required fields are missing
/// - transaction type or status cannot be parsed
pub struct Csv;

// ─────────────────────────────────────────────────────────────────────────────
// Trait implementations
// ─────────────────────────────────────────────────────────────────────────────
impl Decoder for Csv {
    fn decode<R: Read>(&self, r: &mut R) -> Result<Vec<Transaction>, ReaderError> {
        let mut txs = Vec::new();
        let mut reader = BufReader::new(r);

        let mut header = String::new();
        reader.read_line(&mut header).map_err(ReaderError::Io)?;
        parse::validate_header(&header)?;

        for (line_no, line) in reader.lines().enumerate() {
            let line = line.map_err(ReaderError::Io)?;
            if parse::is_blank(&line) {
                continue;
            }

            let tx = parse::parse_line(&line, line_no)?;
            txs.push(tx);
        }

        Ok(txs)
    }
}

impl Encoder for Csv {
    fn encode<W: Write>(&self, txs: &[Transaction], w: &mut W) -> Result<(), WriterError> {
        writeln!(w, "{}", schema::FIELDS_NAMES.join(",")).map_err(WriterError::Io)?;
        for tx in txs {
            writeln!(
                w,
                "{},{},{},{},{},{},{},{}",
                tx.tx_id,
                tx.tx_type,
                tx.from_user_id,
                tx.to_user_id,
                tx.amount,
                tx.timestamp,
                tx.status,
                tx.description // уже содержит кавычки
            )
            .map_err(WriterError::Io)?;
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Private helpers
// ─────────────────────────────────────────────────────────────────────────────
mod parse {
    use crate::error::ReaderError;
    use crate::transaction::schema;
    use crate::transaction::{Transaction, TransactionBuilder};

    pub(super) fn is_blank(s: &str) -> bool {
        s.trim().is_empty()
    }

    pub(super) fn validate_header(line: &str) -> Result<(), ReaderError> {
        let cols: Vec<&str> = line.trim().split(',').collect();
        if cols == schema::FIELDS_NAMES {
            Ok(())
        } else {
            Err(ReaderError::InvalidCsvHeader {
                header: line.trim_end().to_string(),
            })
        }
    }

    pub(super) fn parse_line(line: &str, line_no: usize) -> Result<Transaction, ReaderError> {
        let (fields, description) = split_fields(line, line_no)?;

        let mut builder = TransactionBuilder::new();

        builder.set(schema::TX_ID, fields[0], line_no)?;
        builder.set(schema::TX_TYPE, fields[1], line_no)?;
        builder.set(schema::FROM_USER_ID, fields[2], line_no)?;
        builder.set(schema::TO_USER_ID, fields[3], line_no)?;
        builder.set(schema::AMOUNT, fields[4], line_no)?;
        builder.set(schema::TIMESTAMP, fields[5], line_no)?;
        builder.set(schema::STATUS, fields[6], line_no)?;

        // Builder ожидает description в кавычках
        let quoted = format!("\"{}\"", description);
        builder.set(schema::DESCRIPTION, &quoted, line_no)?;

        builder.build(line_no)
    }

    fn split_fields(line: &str, line_no: usize) -> Result<([&str; 7], &str), ReaderError> {
        let line = line.trim();

        let start = line.find('"').ok_or_else(|| ReaderError::InvalidRow {
            line_no,
            reason: "missing opening quote for DESCRIPTION".to_string(),
        })?;

        let end = line.rfind('"').ok_or_else(|| ReaderError::InvalidRow {
            line_no,
            reason: "missing closing quote for DESCRIPTION".to_string(),
        })?;

        if end <= start {
            return Err(ReaderError::InvalidRow {
                line_no,
                reason: "broken quotes for DESCRIPTION".to_string(),
            });
        }

        let description = &line[start + 1..end];
        let prefix = line[..start].trim_end_matches(',');

        let parts: Vec<&str> = prefix.split(',').collect();
        if parts.len() != 7 {
            return Err(ReaderError::InvalidRow {
                line_no,
                reason: format!("expected 7 fields before DESCRIPTION, got {}", parts.len()),
            });
        }

        Ok((
            [
                parts[0], parts[1], parts[2], parts[3], parts[4], parts[5], parts[6],
            ],
            description,
        ))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_is_blank() {
            assert!(is_blank(""));
            assert!(is_blank("   "));
            assert!(is_blank("\t\n"));
            assert!(!is_blank("x"));
            assert!(!is_blank(" x "));
        }

        #[test]
        fn test_validate_header_ok() {
            let header =
                "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n";
            assert!(validate_header(header).is_ok());
        }

        #[test]
        fn test_validate_header_ok_no_newline() {
            let header =
                "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION";
            assert!(validate_header(header).is_ok());
        }

        #[test]
        fn test_validate_header_wrong() {
            let header = "WRONG,HEADER\n";
            assert!(matches!(
                validate_header(header),
                Err(ReaderError::InvalidCsvHeader { .. })
            ));
        }

        #[test]
        fn test_validate_header_wrong_order() {
            let header =
                "TX_TYPE,TX_ID,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n";
            assert!(matches!(
                validate_header(header),
                Err(ReaderError::InvalidCsvHeader { .. })
            ));
        }

        #[test]
        fn test_split_fields_ok() {
            let line =
                r#"1002,TRANSFER,501,502,15000,1672534800000,FAILURE,"Payment, invoice #123""#;
            let result = split_fields(line, 0);
            assert!(result.is_ok());

            if let Ok((fields, desc)) = result {
                assert_eq!(fields[0], "1002");
                assert_eq!(fields[1], "TRANSFER");
                assert_eq!(fields[6], "FAILURE");
                assert_eq!(desc, "Payment, invoice #123");
            }
        }

        #[test]
        fn test_split_fields_no_quotes() {
            let line = "1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,No quotes";
            assert!(matches!(
                split_fields(line, 0),
                Err(ReaderError::InvalidRow { .. })
            ));
        }

        #[test]
        fn test_split_fields_missing_closing_quote() {
            let line = r#"1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,"No closing"#;

            assert!(matches!(
                split_fields(line, 0),
                Err(ReaderError::InvalidRow { .. })
            ));
        }

        #[test]
        fn test_split_fields_wrong_field_count() {
            let line = r#"1001,DEPOSIT,0,501,50000,"Missing two fields""#;
            let result = split_fields(line, 5);

            if let Err(ReaderError::InvalidRow { line_no, reason }) = result {
                assert_eq!(line_no, 5);
                assert!(reason.contains("expected 7"));
            } else {
                panic!("expected InvalidRow");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{TxStatus, TxType};
    use std::io::Cursor;

    // ─────────────────────────────────────────────────────────────────────
    // Decode: Success cases
    // ─────────────────────────────────────────────────────────────────────
    #[test]
    fn decode_single_row() {
        let data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,"Initial funding"
"#;

        let result = Csv.decode(&mut data.as_bytes());
        assert!(result.is_ok());

        if let Ok(txs) = result {
            assert_eq!(txs.len(), 1);
            assert_eq!(txs[0].tx_id, 1001);
            assert_eq!(txs[0].tx_type, TxType::Deposit);
            assert_eq!(txs[0].status, TxStatus::Success);
        }
    }

    #[test]
    fn decode_multiple_rows() {
        let data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,"First"
1002,TRANSFER,501,502,15000,1672534800000,FAILURE,"Second"
1003,WITHDRAWAL,502,0,1000,1672538400000,PENDING,"Third"
"#;

        let result = Csv.decode(&mut data.as_bytes());
        assert!(result.is_ok());

        if let Ok(txs) = result {
            assert_eq!(txs.len(), 3);
            assert_eq!(txs[0].tx_type, TxType::Deposit);
            assert_eq!(txs[1].tx_type, TxType::Transfer);
            assert_eq!(txs[2].tx_type, TxType::Withdrawal);
        }
    }

    #[test]
    fn decode_description_with_comma() {
        let data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,"Payment for services, invoice #123"
"#;

        let result = Csv.decode(&mut data.as_bytes());
        assert!(result.is_ok());

        if let Ok(txs) = result {
            assert_eq!(txs[0].description, "\"Payment for services, invoice #123\"");
        }
    }

    #[test]
    fn decode_with_empty_lines() {
        let data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,"First"

1002,TRANSFER,501,502,15000,1672534800000,SUCCESS,"Second"

"#;

        let result = Csv.decode(&mut data.as_bytes());
        assert!(result.is_ok());

        if let Ok(txs) = result {
            assert_eq!(txs.len(), 2);
        }
    }

    #[test]
    fn decode_header_only() {
        let data = "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n";

        let result = Csv.decode(&mut data.as_bytes());
        assert!(result.is_ok());

        if let Ok(txs) = result {
            assert!(txs.is_empty());
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Decode: Error cases
    // ─────────────────────────────────────────────────────────────────────
    #[test]
    fn decode_invalid_header() {
        let data = "WRONG,HEADER\n1,2,3\n";

        let result = Csv.decode(&mut data.as_bytes());
        assert!(matches!(result, Err(ReaderError::InvalidCsvHeader { .. })));
    }

    #[test]
    fn decode_empty_file() {
        let data = "";

        let result = Csv.decode(&mut data.as_bytes());
        // Пустой файл = пустой header = ошибка
        assert!(matches!(result, Err(ReaderError::InvalidCsvHeader { .. })));
    }

    #[test]
    fn decode_invalid_tx_type() {
        let data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1001,INVALID,0,501,50000,1672531200000,SUCCESS,"Test"
"#;

        let result = Csv.decode(&mut data.as_bytes());
        assert!(matches!(
            result,
            Err(ReaderError::InvalidFieldValue { field, .. }) if field == "TX_TYPE"
        ));
    }

    #[test]
    fn decode_invalid_status() {
        let data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1001,DEPOSIT,0,501,50000,1672531200000,UNKNOWN,"Test"
"#;

        let result = Csv.decode(&mut data.as_bytes());
        assert!(matches!(
            result,
            Err(ReaderError::InvalidFieldValue { field, .. }) if field == "STATUS"
        ));
    }

    #[test]
    fn decode_invalid_amount() {
        let data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1001,DEPOSIT,0,501,-100,1672531200000,SUCCESS,"Test"
"#;

        let result = Csv.decode(&mut data.as_bytes());
        assert!(matches!(
            result,
            Err(ReaderError::InvalidFieldValue { field, .. }) if field == "AMOUNT"
        ));
    }

    #[test]
    fn decode_missing_quotes() {
        let data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,No quotes here
"#;

        let result = Csv.decode(&mut data.as_bytes());
        assert!(matches!(result, Err(ReaderError::InvalidRow { .. })));
    }

    #[test]
    fn decode_wrong_field_count() {
        let data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1001,DEPOSIT,0,501,"Missing fields"
"#;

        let result = Csv.decode(&mut data.as_bytes());
        assert!(matches!(result, Err(ReaderError::InvalidRow { .. })));
    }

    // ─────────────────────────────────────────────────────────────────────
    // Encode
    // ─────────────────────────────────────────────────────────────────────
    #[test]
    fn encode_single_transaction() {
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

        let output = String::from_utf8(buffer.into_inner());
        assert!(output.is_ok());

        if let Ok(s) = output {
            assert!(s.starts_with("TX_ID,TX_TYPE,"));
            assert!(s.contains("1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,\"Test\""));
        }
    }

    #[test]
    fn encode_empty_list() {
        let txs: Vec<Transaction> = vec![];

        let mut buffer = Cursor::new(Vec::new());
        let result = Csv.encode(&txs, &mut buffer);
        assert!(result.is_ok());

        let output = String::from_utf8(buffer.into_inner());
        assert!(output.is_ok());

        if let Ok(s) = output {
            // Только header
            assert!(s.starts_with("TX_ID,TX_TYPE,"));
            assert_eq!(s.lines().count(), 1);
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Roundtrip
    // ─────────────────────────────────────────────────────────────────────
    #[test]
    fn encode_decode_roundtrip() {
        let original = vec![
            Transaction {
                tx_id: 1001,
                tx_type: TxType::Deposit,
                from_user_id: 0,
                to_user_id: 501,
                amount: 50000,
                timestamp: 1672531200000,
                status: TxStatus::Success,
                description: "\"Initial funding\"".to_string(),
            },
            Transaction {
                tx_id: 1002,
                tx_type: TxType::Transfer,
                from_user_id: 501,
                to_user_id: 502,
                amount: 15000,
                timestamp: 1672534800000,
                status: TxStatus::Failure,
                description: "\"Payment, invoice\"".to_string(),
            },
        ];

        let mut buffer = Cursor::new(Vec::new());
        let encode_result = Csv.encode(&original, &mut buffer);
        assert!(encode_result.is_ok());

        buffer.set_position(0);

        let decode_result = Csv.decode(&mut buffer);
        assert!(decode_result.is_ok());

        if let Ok(decoded) = decode_result {
            assert_eq!(original, decoded);
        }
    }
}

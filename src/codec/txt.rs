use crate::codec::txt::parse::parse_kv;
use crate::codec::{Decoder, Encoder};
use crate::error::{ReaderError, WriterError};
use crate::schema;
use crate::transaction::{Transaction, TransactionBuilder};
use std::io::{BufRead, BufReader, Read, Write};

// ─────────────────────────────────────────────────────────────────────────────
// Public types
// ─────────────────────────────────────────────────────────────────────────────
pub struct Txt;

// ─────────────────────────────────────────────────────────────────────────────
// Trait implementations
// ─────────────────────────────────────────────────────────────────────────────
impl Decoder for Txt {
    fn decode<R: Read>(&self, r: &mut R) -> Result<Vec<Transaction>, ReaderError> {
        let mut txs = Vec::new();
        let reader = BufReader::new(r);
        let mut builder = TransactionBuilder::new();
        let mut tx_start_line_no: usize = 0;
        let mut has_fields = false;

        for (line_no, line) in reader.lines().enumerate() {
            let line = line.map_err(ReaderError::Io)?;
            let field = line.trim();

            if field.starts_with('#') {
                continue;
            }

            if field.is_empty() {
                if has_fields {
                    txs.push(builder.build(tx_start_line_no)?);
                    builder = TransactionBuilder::new();
                    has_fields = false;
                }
                continue;
            }

            if !has_fields {
                tx_start_line_no = line_no;
                has_fields = true;
            }

            let (key, value) = parse_kv(field).ok_or_else(|| ReaderError::InvalidRow {
                line_no,
                reason: field.to_string(),
            })?;

            builder.set(&key, &value, line_no)?;
        }

        if has_fields {
            txs.push(builder.build(tx_start_line_no)?);
        }

        Ok(txs)
    }
}

impl Encoder for Txt {
    fn encode<W: Write>(&self, txs: &[Transaction], w: &mut W) -> Result<(), WriterError> {
        for tx in txs {
            writeln!(
                w,
                "{}:{}\n{}:{}\n{}:{}\n{}:{}\n{}:{}\n{}:{}\n{}:{}\n{}:{}\n\n",
                schema::TX_ID,
                tx.tx_id,
                schema::TX_TYPE,
                tx.tx_type,
                schema::FROM_USER_ID,
                tx.from_user_id,
                schema::TO_USER_ID,
                tx.to_user_id,
                schema::AMOUNT,
                tx.amount,
                schema::TIMESTAMP,
                tx.timestamp,
                schema::STATUS,
                tx.status,
                schema::DESCRIPTION,
                tx.description
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
    pub(super) fn parse_kv(line: &str) -> Option<(String, String)> {
        let (key, value) = line.split_once(':')?;
        let key = key.trim();
        let value = value.trim();

        if key.is_empty() || value.is_empty() {
            return None;
        }
        Some((key.to_string(), value.to_string()))
    }

    #[cfg(test)]
    mod tests {
        use super::parse_kv;

        #[test]
        fn test_parse_kv_ok() {
            assert_eq!(
                parse_kv("TX_ID: 123"),
                Some(("TX_ID".to_string(), "123".to_string()))
            );
            assert_eq!(
                parse_kv("DESCRIPTION: \"ABC\""),
                Some(("DESCRIPTION".to_string(), "\"ABC\"".to_string()))
            );
            assert_eq!(
                parse_kv("DESCRIPTION: \"ABC: CDE\""),
                Some(("DESCRIPTION".to_string(), "\"ABC: CDE\"".to_string()))
            );
        }

        #[test]
        fn test_parse_kv_invalid() {
            assert_eq!(parse_kv("TX_ID"), None);
            assert_eq!(parse_kv(":ABC"), None);
            assert_eq!(parse_kv("ABC:"), None);
            assert_eq!(parse_kv(":"), None);
            assert_eq!(parse_kv(""), None);
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
    fn decode_single_transaction() {
        let data = r#"TX_ID: 123
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 456
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Test deposit"
"#;

        let result = Txt.decode(&mut data.as_bytes());
        assert!(result.is_ok());

        if let Ok(txs) = result {
            assert_eq!(txs.len(), 1);
            assert_eq!(txs[0].tx_id, 123);
            assert_eq!(txs[0].tx_type, TxType::Deposit);
            assert_eq!(txs[0].from_user_id, 0);
            assert_eq!(txs[0].to_user_id, 456);
            assert_eq!(txs[0].amount, 10000);
            assert_eq!(txs[0].timestamp, 1633036800000);
            assert_eq!(txs[0].status, TxStatus::Success);
            assert_eq!(txs[0].description, "\"Test deposit\"");
        }
    }

    #[test]
    fn decode_multiple_transactions() {
        let data = r#"TX_ID: 1
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 100
AMOUNT: 1000
TIMESTAMP: 1000000
STATUS: SUCCESS
DESCRIPTION: "First"

TX_ID: 2
TX_TYPE: TRANSFER
FROM_USER_ID: 100
TO_USER_ID: 200
AMOUNT: 500
TIMESTAMP: 2000000
STATUS: PENDING
DESCRIPTION: "Second"

TX_ID: 3
TX_TYPE: WITHDRAWAL
FROM_USER_ID: 200
TO_USER_ID: 0
AMOUNT: 100
TIMESTAMP: 3000000
STATUS: FAILURE
DESCRIPTION: "Third"
"#;

        let result = Txt.decode(&mut data.as_bytes());
        assert!(result.is_ok());

        if let Ok(txs) = result {
            assert_eq!(txs.len(), 3);
            assert_eq!(txs[0].tx_id, 1);
            assert_eq!(txs[0].tx_type, TxType::Deposit);
            assert_eq!(txs[1].tx_id, 2);
            assert_eq!(txs[1].tx_type, TxType::Transfer);
            assert_eq!(txs[1].status, TxStatus::Pending);
            assert_eq!(txs[2].tx_id, 3);
            assert_eq!(txs[2].tx_type, TxType::Withdrawal);
            assert_eq!(txs[2].status, TxStatus::Failure);
        }
    }

    #[test]
    fn decode_with_comments() {
        let data = r#"# This is a comment
TX_ID: 123
# Another comment
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 456
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Test"
# Trailing comment
"#;

        let result = Txt.decode(&mut data.as_bytes());
        assert!(result.is_ok());

        if let Ok(txs) = result {
            assert_eq!(txs.len(), 1);
            assert_eq!(txs[0].tx_id, 123);
        }
    }

    #[test]
    fn decode_with_multiple_empty_lines() {
        let data = r#"TX_ID: 1
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 100
AMOUNT: 1000
TIMESTAMP: 1000000
STATUS: SUCCESS
DESCRIPTION: "First"



TX_ID: 2
TX_TYPE: TRANSFER
FROM_USER_ID: 100
TO_USER_ID: 200
AMOUNT: 500
TIMESTAMP: 2000000
STATUS: SUCCESS
DESCRIPTION: "Second"
"#;

        let result = Txt.decode(&mut data.as_bytes());
        assert!(result.is_ok());

        if let Ok(txs) = result {
            assert_eq!(txs.len(), 2);
        }
    }

    #[test]
    fn decode_fields_any_order() {
        let data = r#"DESCRIPTION: "Reversed order"
STATUS: SUCCESS
TIMESTAMP: 1633036800000
AMOUNT: 10000
TO_USER_ID: 456
FROM_USER_ID: 0
TX_TYPE: DEPOSIT
TX_ID: 123
"#;

        let result = Txt.decode(&mut data.as_bytes());
        assert!(result.is_ok());

        if let Ok(txs) = result {
            assert_eq!(txs.len(), 1);
            assert_eq!(txs[0].tx_id, 123);
            assert_eq!(txs[0].tx_type, TxType::Deposit);
        }
    }

    #[test]
    fn decode_empty_file() {
        let data = "";
        let result = Txt.decode(&mut data.as_bytes());
        assert!(result.is_ok());

        if let Ok(txs) = result {
            assert!(txs.is_empty());
        }
    }

    #[test]
    fn decode_only_comments_and_empty_lines() {
        let data = r#"# Comment 1
# Comment 2

# Comment 3
"#;

        let result = Txt.decode(&mut data.as_bytes());
        assert!(result.is_ok());

        if let Ok(txs) = result {
            assert!(txs.is_empty());
        }
    }

    #[test]
    fn decode_no_trailing_newline() {
        let data = "TX_ID: 123
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 456
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: \"Test\"";

        let result = Txt.decode(&mut data.as_bytes());
        assert!(result.is_ok());

        if let Ok(txs) = result {
            assert_eq!(txs.len(), 1);
        }
    }

    #[test]
    fn decode_description_with_colon() {
        let data = r#"TX_ID: 123
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 456
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Key: Value inside"
"#;

        let result = Txt.decode(&mut data.as_bytes());
        assert!(result.is_ok());

        if let Ok(txs) = result {
            assert_eq!(txs[0].description, "\"Key: Value inside\"");
        }
    }

    #[test]
    fn decode_with_leading_empty_lines() {
        let data = r#"

TX_ID: 123
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 456
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Test"
"#;

        let result = Txt.decode(&mut data.as_bytes());
        assert!(result.is_ok());

        if let Ok(txs) = result {
            assert_eq!(txs.len(), 1);
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Decode: Error cases
    // ─────────────────────────────────────────────────────────────────────
    #[test]
    fn decode_missing_field() {
        let data = r#"TX_ID: 123
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 456
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
"#;

        let result = Txt.decode(&mut data.as_bytes());
        assert!(result.is_err());

        if let Err(ReaderError::MissingFields { fields, .. }) = result {
            assert!(fields.contains(&"DESCRIPTION".to_string()));
        } else {
            panic!("expected MissingFields");
        }
    }

    #[test]
    fn decode_missing_multiple_fields() {
        let data = r#"TX_ID: 123
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
"#;

        let result = Txt.decode(&mut data.as_bytes());
        assert!(result.is_err());

        if let Err(ReaderError::MissingFields { fields, .. }) = result {
            assert!(fields.contains(&"TO_USER_ID".to_string()));
            assert!(fields.contains(&"AMOUNT".to_string()));
            assert!(fields.contains(&"TIMESTAMP".to_string()));
            assert!(fields.contains(&"STATUS".to_string()));
            assert!(fields.contains(&"DESCRIPTION".to_string()));
        } else {
            panic!("expected MissingFields");
        }
    }

    #[test]
    fn decode_duplicate_field() {
        let data = r#"TX_ID: 123
TX_ID: 456
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 456
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Test"
"#;

        let result = Txt.decode(&mut data.as_bytes());
        assert!(result.is_err());

        if let Err(ReaderError::DuplicateField { field, .. }) = result {
            assert_eq!(field, "TX_ID");
        } else {
            panic!("expected DuplicateField");
        }
    }

    #[test]
    fn decode_unknown_field() {
        let data = r#"TX_ID: 123
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 456
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Test"
UNKNOWN_FIELD: value
"#;

        let result = Txt.decode(&mut data.as_bytes());
        assert!(result.is_err());

        if let Err(ReaderError::UnknownField { field, .. }) = result {
            assert_eq!(field, "UNKNOWN_FIELD");
        } else {
            panic!("expected UnknownField");
        }
    }

    #[test]
    fn decode_invalid_tx_id() {
        let data = r#"TX_ID: not_a_number
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 456
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Test"
"#;

        let result = Txt.decode(&mut data.as_bytes());
        assert!(result.is_err());

        assert!(matches!(
            result,
            Err(ReaderError::InvalidFieldValue { field, .. }) if field == "TX_ID"
        ));
    }

    #[test]
    fn decode_invalid_tx_type() {
        let data = r#"TX_ID: 123
TX_TYPE: INVALID
FROM_USER_ID: 0
TO_USER_ID: 456
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Test"
"#;

        let result = Txt.decode(&mut data.as_bytes());
        assert!(result.is_err());

        assert!(matches!(
            result,
            Err(ReaderError::InvalidFieldValue { field, .. }) if field == "TX_TYPE"
        ));
    }

    #[test]
    fn decode_invalid_status() {
        let data = r#"TX_ID: 123
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 456
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: UNKNOWN
DESCRIPTION: "Test"
"#;

        let result = Txt.decode(&mut data.as_bytes());
        assert!(result.is_err());

        assert!(matches!(
            result,
            Err(ReaderError::InvalidFieldValue { field, .. }) if field == "STATUS"
        ));
    }

    #[test]
    fn decode_invalid_amount_negative() {
        let data = r#"TX_ID: 123
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 456
AMOUNT: -100
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Test"
"#;

        let result = Txt.decode(&mut data.as_bytes());
        assert!(result.is_err());

        assert!(matches!(
            result,
            Err(ReaderError::InvalidFieldValue { field, .. }) if field == "AMOUNT"
        ));
    }

    #[test]
    fn decode_description_missing_quotes() {
        let data = r#"TX_ID: 123
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 456
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: No quotes here
"#;

        let result = Txt.decode(&mut data.as_bytes());
        assert!(result.is_err());

        assert!(matches!(
            result,
            Err(ReaderError::InvalidRow { reason, .. }) if reason.contains("DESCRIPTION")
        ));
    }

    #[test]
    fn decode_description_missing_closing_quote() {
        let data = r#"TX_ID: 123
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 456
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "No closing quote
"#;

        let result = Txt.decode(&mut data.as_bytes());
        assert!(result.is_err());
    }

    #[test]
    fn decode_invalid_line_format() {
        let data = r#"TX_ID: 123
TX_TYPE: DEPOSIT
this is not a valid line
FROM_USER_ID: 0
TO_USER_ID: 456
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Test"
"#;

        let result = Txt.decode(&mut data.as_bytes());
        assert!(result.is_err());

        assert!(matches!(result, Err(ReaderError::InvalidRow { .. })));
    }

    #[test]
    fn decode_error_in_second_transaction() {
        let data = r#"TX_ID: 1
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 100
AMOUNT: 1000
TIMESTAMP: 1000000
STATUS: SUCCESS
DESCRIPTION: "First"

TX_ID: invalid
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 100
AMOUNT: 1000
TIMESTAMP: 1000000
STATUS: SUCCESS
DESCRIPTION: "Second"
"#;

        let result = Txt.decode(&mut data.as_bytes());
        assert!(result.is_err());

        assert!(matches!(
            result,
            Err(ReaderError::InvalidFieldValue { field, .. }) if field == "TX_ID"
        ));
    }

    // ─────────────────────────────────────────────────────────────────────
    // Encode: Success cases
    // ─────────────────────────────────────────────────────────────────────
    #[test]
    fn encode_single_transaction() {
        let txs = vec![Transaction {
            tx_id: 123,
            tx_type: TxType::Deposit,
            from_user_id: 0,
            to_user_id: 456,
            amount: 10000,
            timestamp: 1633036800000,
            status: TxStatus::Success,
            description: "\"Test deposit\"".to_string(),
        }];

        let mut buffer = Cursor::new(Vec::new());
        let result = Txt.encode(&txs, &mut buffer);
        assert!(result.is_ok());

        let output = String::from_utf8(buffer.into_inner());
        assert!(output.is_ok());

        if let Ok(s) = output {
            assert!(s.contains("TX_ID:123"));
            assert!(s.contains("TX_TYPE:DEPOSIT"));
            assert!(s.contains("AMOUNT:10000"));
            assert!(s.contains("DESCRIPTION:\"Test deposit\""));
        }
    }

    #[test]
    fn encode_empty_list() {
        let txs: Vec<Transaction> = vec![];

        let mut buffer = Cursor::new(Vec::new());
        let result = Txt.encode(&txs, &mut buffer);
        assert!(result.is_ok());

        let output = String::from_utf8(buffer.into_inner());
        assert!(output.is_ok());

        if let Ok(s) = output {
            assert!(s.is_empty());
        }
    }

    #[test]
    fn encode_multiple_transactions() {
        let txs = vec![
            Transaction {
                tx_id: 1,
                tx_type: TxType::Deposit,
                from_user_id: 0,
                to_user_id: 100,
                amount: 1000,
                timestamp: 1000000,
                status: TxStatus::Success,
                description: "\"First\"".to_string(),
            },
            Transaction {
                tx_id: 2,
                tx_type: TxType::Transfer,
                from_user_id: 100,
                to_user_id: 200,
                amount: 500,
                timestamp: 2000000,
                status: TxStatus::Pending,
                description: "\"Second\"".to_string(),
            },
        ];

        let mut buffer = Cursor::new(Vec::new());
        let result = Txt.encode(&txs, &mut buffer);
        assert!(result.is_ok());

        let output = String::from_utf8(buffer.into_inner());
        assert!(output.is_ok());

        if let Ok(s) = output {
            assert!(s.contains("TX_ID:1"));
            assert!(s.contains("TX_ID:2"));
            assert!(s.contains("TX_TYPE:DEPOSIT"));
            assert!(s.contains("TX_TYPE:TRANSFER"));
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
                description: "\"Payment: invoice #123\"".to_string(),
            },
        ];

        let mut buffer = Cursor::new(Vec::new());
        let encode_result = Txt.encode(&original, &mut buffer);
        assert!(encode_result.is_ok());

        buffer.set_position(0);

        let decode_result = Txt.decode(&mut buffer);
        assert!(decode_result.is_ok());

        if let Ok(decoded) = decode_result {
            assert_eq!(original, decoded);
        }
    }

    #[test]
    fn encode_decode_all_tx_types() {
        for tx_type in [TxType::Deposit, TxType::Transfer, TxType::Withdrawal] {
            let original = vec![Transaction {
                tx_id: 1,
                tx_type,
                from_user_id: 0,
                to_user_id: 100,
                amount: 1000,
                timestamp: 1000000,
                status: TxStatus::Success,
                description: "\"Test\"".to_string(),
            }];

            let mut buffer = Cursor::new(Vec::new());
            assert!(Txt.encode(&original, &mut buffer).is_ok());

            buffer.set_position(0);

            let decode_result = Txt.decode(&mut buffer);
            assert!(decode_result.is_ok());

            if let Ok(decoded) = decode_result {
                assert_eq!(decoded[0].tx_type, tx_type);
            }
        }
    }

    #[test]
    fn encode_decode_all_statuses() {
        for status in [TxStatus::Success, TxStatus::Failure, TxStatus::Pending] {
            let original = vec![Transaction {
                tx_id: 1,
                tx_type: TxType::Deposit,
                from_user_id: 0,
                to_user_id: 100,
                amount: 1000,
                timestamp: 1000000,
                status,
                description: "\"Test\"".to_string(),
            }];

            let mut buffer = Cursor::new(Vec::new());
            assert!(Txt.encode(&original, &mut buffer).is_ok());

            buffer.set_position(0);

            let decode_result = Txt.decode(&mut buffer);
            assert!(decode_result.is_ok());

            if let Ok(decoded) = decode_result {
                assert_eq!(decoded[0].status, status);
            }
        }
    }
}
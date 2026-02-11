use crate::error::{ReaderError, WriterError};
use crate::transaction::{Transaction, TransactionDecoder, TransactionEncoder, TxStatus, TxType};
use std::io::{BufRead, BufReader, Read, Write};

pub struct Csv;

// ─────────────────────────────────────────────────────────────────────────────
// Csv decoder
// ─────────────────────────────────────────────────────────────────────────────
impl TransactionDecoder for Csv {
    fn decode_all<R: Read>(&self, r: &mut R) -> Result<Vec<Transaction>, ReaderError> {
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

            let (data, description) = parse::split_tx_line(&line, line_no)?;
            let [
                tx_id_str,
                tx_type_str,
                from_str,
                to_str,
                amount_str,
                ts_str,
                status_str,
            ] = data;

            let tx_id = parse::parse_u64_field(tx_id_str, field::TX_ID, line_no)?;
            let from_user_id = parse::parse_u64_field(from_str, field::FROM_USER_ID, line_no)?;
            let to_user_id = parse::parse_u64_field(to_str, field::TO_USER_ID, line_no)?;
            let amount = parse::parse_u64_field(amount_str, field::AMOUNT, line_no)?;
            let timestamp = parse::parse_u64_field(ts_str, field::TIMESTAMP, line_no)?;

            let tx_type =
                TxType::parse(tx_type_str).ok_or_else(|| ReaderError::InvalidFieldValue {
                    line_no,
                    field: field::TX_TYPE.to_string(),
                    value: tx_type_str.to_string(),
                })?;

            let status =
                TxStatus::parse(status_str).ok_or_else(|| ReaderError::InvalidFieldValue {
                    line_no,
                    field: field::STATUS.to_string(),
                    value: status_str.to_string(),
                })?;

            txs.push(Transaction {
                tx_id,
                tx_type,
                from_user_id,
                to_user_id,
                amount,
                timestamp,
                status,
                description: description.to_string(),
            });
        }

        Ok(txs)
    }
}

impl TransactionEncoder for Csv {
    fn encode_all<W: Write>(&self, txs: &Vec<Transaction>, w: &mut W) -> Result<(), WriterError> {
        todo!()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Field name constants
// ─────────────────────────────────────────────────────────────────────────────
mod field {
    pub const TX_ID: &str = "TX_ID";
    pub const TX_TYPE: &str = "TX_TYPE";
    pub const FROM_USER_ID: &str = "FROM_USER_ID";
    pub const TO_USER_ID: &str = "TO_USER_ID";
    pub const AMOUNT: &str = "AMOUNT";
    pub const TIMESTAMP: &str = "TIMESTAMP";
    pub const STATUS: &str = "STATUS";
    pub const DESCRIPTION: &str = "DESCRIPTION";
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal parsing helpers
// ─────────────────────────────────────────────────────────────────────────────
mod parse {
    use super::field;
    use crate::error::ReaderError;

    pub(crate) const EXPECTED_HEADER: [&str; 8] = [
        field::TX_ID,
        field::TX_TYPE,
        field::FROM_USER_ID,
        field::TO_USER_ID,
        field::AMOUNT,
        field::TIMESTAMP,
        field::STATUS,
        field::DESCRIPTION,
    ];

    pub(crate) fn is_blank(s: &str) -> bool {
        s.trim().is_empty()
    }

    pub(crate) fn validate_header(line: &str) -> Result<(), ReaderError> {
        let cols: Vec<&str> = line.trim().split(',').collect();
        if cols == EXPECTED_HEADER {
            Ok(())
        } else {
            Err(ReaderError::InvalidCsvHeader {
                header: line.trim_end().to_string(),
            })
        }
    }

    pub(crate) fn parse_u64_field(
        value: &str,
        field_name: &str,
        line_no: usize,
    ) -> Result<u64, ReaderError> {
        value
            .trim()
            .parse()
            .map_err(|_| ReaderError::InvalidFieldValue {
                line_no,
                field: field_name.to_string(),
                value: value.to_string(),
            })
    }

    pub(crate) fn split_tx_line(
        line: &str,
        line_no: usize,
    ) -> Result<([&str; 7], &str), ReaderError> {
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

    // ─────────────────────────────────────────────────────────────────────────
    // Tests
    // ─────────────────────────────────────────────────────────────────────────
    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_is_blank() {
            assert!(is_blank(""));
            assert!(is_blank("   "));
            assert!(is_blank("\n\t "));
            assert!(!is_blank("x"));
            assert!(!is_blank("  x  "));
        }

        #[test]
        fn test_validate_header_ok() {
            let header =
                "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n";
            assert!(validate_header(header).is_ok());
        }

        #[test]
        fn test_validate_header_wrong() {
            let header = "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCR\n";
            assert!(matches!(
                validate_header(header),
                Err(ReaderError::InvalidCsvHeader { .. })
            ));
        }

        #[test]
        fn test_split_tx_line_ok_with_comma_in_description() {
            let line = r#"1002,TRANSFER,501,502,15000,1672534800000,FAILURE,"Payment for services, invoice #123""#;
            let (data, desc) = split_tx_line(line, 0).unwrap();

            assert_eq!(data[0], "1002");
            assert_eq!(data[1], "TRANSFER");
            assert_eq!(data[2], "501");
            assert_eq!(data[3], "502");
            assert_eq!(data[4], "15000");
            assert_eq!(data[5], "1672534800000");
            assert_eq!(data[6], "FAILURE");
            assert_eq!(desc, "Payment for services, invoice #123");
        }

        #[test]
        fn test_split_tx_line_err_no_quotes() {
            let line = "1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,Initial account funding";
            let err = split_tx_line(line, 0).unwrap_err();
            assert!(matches!(err, ReaderError::InvalidRow { .. }));
        }

        #[test]
        fn test_split_tx_line_err_missing_last_quote() {
            let line = r#"1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,"Initial account funding"#;
            let err = split_tx_line(line, 0).unwrap_err();
            assert!(matches!(err, ReaderError::InvalidRow { .. }));
        }

        #[test]
        fn test_split_tx_line_err_wrong_data_field_count() {
            let line = r#"1001,DEPOSIT,0,501,50000,1672531200000,"Initial account funding""#;
            let err = split_tx_line(line, 0).unwrap_err();
            assert!(matches!(err, ReaderError::InvalidRow { .. }));
        }

        #[test]
        fn test_parse_u64_field_ok() {
            assert_eq!(parse_u64_field("123", "TEST", 0).unwrap(), 123);
            assert_eq!(parse_u64_field("  456  ", "TEST", 0).unwrap(), 456);
        }

        #[test]
        fn test_parse_u64_field_err() {
            let err = parse_u64_field("abc", "TEST", 5).unwrap_err();
            assert!(matches!(
                err,
                ReaderError::InvalidFieldValue { line_no: 5, .. }
            ));
        }
    }
}

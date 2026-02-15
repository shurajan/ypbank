use crate::error::{ReaderError, WriterError};
use crate::transaction::{Transaction, TransactionBuilder};
use crate::{Decoder, Encoder, schema};
use std::io::{BufRead, BufReader, Read, Write};

// ─────────────────────────────────────────────────────────────────────────────
// Public types
// ─────────────────────────────────────────────────────────────────────────────
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
    use crate::schema;
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
            assert!(!is_blank("x"));
        }

        #[test]
        fn test_validate_header_ok() {
            let header =
                "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n";
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
        fn test_split_fields_ok() {
            let line =
                r#"1002,TRANSFER,501,502,15000,1672534800000,FAILURE,"Payment, invoice #123""#;
            let (fields, desc) = split_fields(line, 0).unwrap();

            assert_eq!(fields[0], "1002");
            assert_eq!(fields[1], "TRANSFER");
            assert_eq!(fields[6], "FAILURE");
            assert_eq!(desc, "Payment, invoice #123");
        }

        #[test]
        fn test_split_fields_no_quotes() {
            let line = "1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,No quotes";
            assert!(matches!(
                split_fields(line, 0),
                Err(ReaderError::InvalidRow { .. })
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{TxStatus, TxType};
    use std::io::Cursor;

    #[test]
    fn decode_valid_csv() {
        let csv = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,"Initial account funding"
1002,TRANSFER,501,502,15000,1672534800000,FAILURE,"Payment for services, invoice #123"
"#;

        let txs = Csv.decode(&mut csv.as_bytes()).unwrap();

        assert_eq!(txs.len(), 2);
        assert_eq!(txs[0].tx_id, 1001);
        assert_eq!(txs[0].tx_type, TxType::Deposit);
        assert_eq!(txs[1].description, "\"Payment for services, invoice #123\"");
    }

    #[test]
    fn encode_decode_roundtrip() {
        let original = vec![Transaction {
            tx_id: 1001,
            tx_type: TxType::Deposit,
            from_user_id: 0,
            to_user_id: 501,
            amount: 50000,
            timestamp: 1672531200000,
            status: TxStatus::Success,
            description: "\"Initial funding\"".to_string(),
        }];

        let mut buffer = Cursor::new(Vec::new());
        Csv.encode(&original, &mut buffer).unwrap();
        buffer.set_position(0);
        let decoded = Csv.decode(&mut buffer).unwrap();

        assert_eq!(original, decoded);
    }
}

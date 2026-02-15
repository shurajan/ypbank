use crate::codec::txt::parse::parse_kv;
use crate::codec::{Decoder, Encoder};
use crate::error::{ReaderError, WriterError};
use crate::transaction::{Transaction, TransactionBuilder};
use std::io::{BufRead, BufReader, Read, Write};
use crate::schema;

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
                schema::TX_ID, tx.tx_id,
                schema::TX_TYPE,tx.tx_type,
                schema::FROM_USER_ID,tx.from_user_id,
                schema::TO_USER_ID,tx.to_user_id,
                schema::AMOUNT,tx.amount,
                schema::TIMESTAMP,tx.timestamp,
                schema::STATUS,tx.status,
                schema::DESCRIPTION,tx.description
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
    #[test]
    fn encode_decode_roundtrip() {
        let original = vec![Transaction {
            tx_id: 1001,
            tx_type:TxType::Deposit,
            from_user_id: 0,
            to_user_id: 501,
            amount: 50000,
            timestamp: 1672531200000,
            status: TxStatus::Success,
            description: "\"Initial funding\"".to_string(),
        }];

        let mut buffer = Cursor::new(Vec::new());
        Txt.encode(&original, &mut buffer).unwrap();
        buffer.set_position(0);
        let decoded = Txt.decode(&mut buffer).unwrap();

        assert_eq!(original, decoded);
    }
}

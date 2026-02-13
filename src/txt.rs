use std::collections::HashMap;
use crate::error::{ReaderError, WriterError};
use crate::transaction::{Transaction, TransactionDecoder, TransactionEncoder, TxStatus, TxType};
use std::io::{BufRead, BufReader, Read, Write};
use crate::schema;

// ─────────────────────────────────────────────────────────────────────────────
// Public types
// ─────────────────────────────────────────────────────────────────────────────
pub struct Txt;

// ─────────────────────────────────────────────────────────────────────────────
// Trait implementations
// ─────────────────────────────────────────────────────────────────────────────
impl TransactionDecoder for Txt {
    fn decode_all<R: Read>(&self, r: &mut R) -> Result<Vec<Transaction>, ReaderError> {
        let mut txs = Vec::new();
        let mut reader = BufReader::new(r);

        for (line_no, line) in reader.lines().enumerate() {
            let fields:HashMap<String, String> = HashMap::new();

            if line.unwrap().trim().is_empty() && !fields.is_empty() {
               todo!()
            } else { continue; }

            let line = line.map_err(ReaderError::Io)?;
            println!("{}", line);
        }

        Ok(txs)
    }
}


impl TransactionEncoder for Txt {
    fn encode_all<W: Write>(&self, txs: &Vec<Transaction>, w: &mut W) -> Result<(), WriterError> {
        todo!()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Private helpers
// ─────────────────────────────────────────────────────────────────────────────
mod parse {
    use crate::Transaction;

    pub(super) fn parse_kv(line: &str) -> Option<(String, String)> {
        let key_val: Vec<&str> = line.splitn(2, ':').collect();
        if key_val.len() == 2 {
            let key = key_val[0].trim().to_string();
            let value = key_val[1].trim().to_string();
            if key.is_empty() || value.is_empty() {
                return None;
            }
            Some((key, value))
        } else {
            None
        }
    }

    mod test {
        use crate::txt::parse::parse_kv;

        #[test]
        fn test_parse_kv_ok() {
            let line = "TX_ID: 123";
            let kv = parse_kv(line).unwrap();
            assert_eq!(kv, ("TX_ID".to_string(), "123".to_string()));

            let line = "DESCRIPTION: \"ABC\"";
            let kv = parse_kv(line).unwrap();
            assert_eq!(kv, ("DESCRIPTION".to_string(), "\"ABC\"".to_string()));

            let line = "DESCRIPTION: \"ABC: CDE\"";
            let kv = parse_kv(line).unwrap();
            assert_eq!(kv, ("DESCRIPTION".to_string(), "\"ABC: CDE\"".to_string()));

            let line = "DESCRIPTION: \"\"";
            let kv = parse_kv(line).unwrap();
            assert_eq!(kv, ("DESCRIPTION".to_string(), "\"\"".to_string()));
        }

        #[test]
        fn test_parse_kv_empty() {
            let line = "TX_ID";
            let kv = parse_kv(line);
            assert_eq!(kv, None);

            let line = ":ABC";
            let kv = parse_kv(line);
            assert_eq!(kv, None);

            let line = "ABC:";
            let kv = parse_kv(line);
            assert_eq!(kv, None);

            let line = ":";
            let kv = parse_kv(line);
            assert_eq!(kv, None);
        }
    }
}

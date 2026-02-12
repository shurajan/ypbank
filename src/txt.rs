use crate::error::{ReaderError, WriterError};
use crate::transaction::{Transaction, TransactionDecoder, TransactionEncoder};
use std::io::{Read, Write};

// ─────────────────────────────────────────────────────────────────────────────
// Public types
// ─────────────────────────────────────────────────────────────────────────────
pub struct Txt;

// ─────────────────────────────────────────────────────────────────────────────
// Trait implementations
// ─────────────────────────────────────────────────────────────────────────────
impl TransactionDecoder for Txt {
    fn decode_all<R: Read>(&self, r: &mut R) -> Result<Vec<Transaction>, ReaderError> {
        todo!()
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

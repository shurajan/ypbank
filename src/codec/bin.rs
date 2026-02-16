use super::{Decoder, Encoder};
use crate::error::{ReaderError, WriterError};
use crate::transaction::Transaction;
use std::io::{BufReader, Cursor, Read, Write};
use crate::codec::bin::parse::read_record;

pub struct Bin;

impl Decoder for Bin {
    fn decode<R: Read>(&self, r: &mut R) -> Result<Vec<Transaction>, ReaderError> {
        let mut reader = BufReader::new(r);

        let mut transactions = Vec::new();

        while let Some(tx) = read_record(&mut reader)? {
            transactions.push(tx);
        }
        Ok(transactions)
    }
}

impl Encoder for Bin {
    fn encode<W: Write>(&self, txs: &[Transaction], w: &mut W) -> Result<(), WriterError> {
        todo!()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Private helpers
// ─────────────────────────────────────────────────────────────────────────────
mod parse {
    use crate::transaction::{TxStatus, TxType};
    use crate::{ReaderError, Transaction};
    use std::io::{ErrorKind, Read};

    pub(super) const MAGIC: [u8; 4] = [0x59, 0x50, 0x42, 0x4E];
    pub(super) const MIN_RECORD_SIZE: u32 = 8 + 1 + 8 + 8 + 8 + 8 + 1 + 4;

    macro_rules! read_be {
        ($r:expr, $ty:ty) => {{
            let mut buf = [0u8; std::mem::size_of::<$ty>()];
            $r.read_exact(&mut buf).map_err(ReaderError::Io)?;
            <$ty>::from_be_bytes(buf)
        }};
    }

    macro_rules! read_string {
        ($r:expr, $len:expr) => {{
            let mut buf = vec![0u8; $len as usize];
            $r.read_exact(&mut buf).map_err(ReaderError::Io)?;
            String::from_utf8_lossy(&buf).into_owned()
        }};
    }

    pub(super) fn read_record<R: Read>(r: &mut R) -> Result<Option<Transaction>, ReaderError> {
        // Magic (с обработкой EOF)
        let mut magic = [0u8; 4];
        match r.read_exact(&mut magic) {
            Ok(()) => {}
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(ReaderError::Io(e)),
        }

        if magic != MAGIC {
            return Err(ReaderError::InvalidMagic);
        }

        // Record size
        let record_size = read_be!(r, u32);
        if record_size < MIN_RECORD_SIZE {
            return Err(ReaderError::InvalidBinaryFormat {
                reason: format!(
                    "wrong record size - {}, should be at least {}",
                    record_size, MIN_RECORD_SIZE
                ),
            });
        }

        // Fields
        let tx_id = read_be!(r, u64);

        let tx_type_raw = read_be!(r, u8);

        let tx_type = TxType::from(tx_type_raw).ok_or_else(|| ReaderError::InvalidBinaryFormat {
            reason: format!("TxType should be 0 = DEPOSIT, 1 = TRANSFER, 2 = WITHDRAWAL. Recieved value - {}",
                            tx_type_raw),
        })?;

        let from_user_id = read_be!(r, u64);
        let to_user_id = read_be!(r, u64);
        let amount = read_be!(r, i64).unsigned_abs();
        let timestamp = read_be!(r, u64);
        let status_raw = read_be!(r, u8);

        let status =
            TxStatus::from(status_raw).ok_or_else(|| ReaderError::InvalidBinaryFormat {
                reason: format!(
                    "TxStatus should be 0 = SUCCESS, 1 = FAILURE, 2 = PENDING. Recieved value - {}",
                    status_raw
                ),
            })?;

        // Description
        let desc_len = read_be!(r, u32);
        let real_size = desc_len
            .checked_add(MIN_RECORD_SIZE)
            .ok_or_else(|| ReaderError::InvalidBinaryFormat {
                reason: "record size overflow".to_string(),
            })?;

        if real_size != record_size {
            return Err(ReaderError::InvalidBinaryFormat {
                reason: format!(
                    "real size of the record ({}) not equal to expected ({})",
                    real_size, record_size
                ),
            });
        }

        if real_size != record_size {
            return Err(ReaderError::InvalidBinaryFormat {
                reason: format!(
                    "real size of the record - {} not equal to expected {}",
                    real_size, record_size
                ),
            });
        }

        let description = read_string!(r, desc_len);



        Ok(Some(Transaction {
            tx_id,
            tx_type,
            from_user_id,
            to_user_id,
            amount,
            timestamp,
            status,
            description,
        }))
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::io::Cursor;

        #[test]
        fn test_parse_ok() {
            let data  = [
                0x59, 0x50, 0x42, 0x4e, // magic = "YPBN"
                0x00, 0x00, 0x00, 0x3f, // record_size = 63

                0x00, 0x03, 0x8d, 0x7e, 0xa4, 0xc6, 0x80, 0x00, // tx_id
                0x00,                                           // tx_type

                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // from_user_id
                0x00, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // to_user_id

                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, // amount = 100
                0x00, 0x00, 0x01, 0x7c, 0x38, 0x94, 0xfa, 0x60, // timestamp

                0x01,                         // status
                0x00, 0x00, 0x00, 0x11,       // desc_len = 17

                // description string (17 bytes)
                0x22, 0x52, 0x65, 0x63, 0x6f, 0x72, 0x64,
                0x20, 0x6e, 0x75, 0x6d, 0x62, 0x65, 0x72,
                0x20, 0x31, 0x22,
            ];

            let mut cursor = Cursor::new(data);
            let result = read_record(&mut cursor).unwrap().unwrap();

            assert_eq!(result.tx_id, 1000000000000000);
            assert_eq!(result.from_user_id, 0);
            assert_eq!(result.to_user_id, 36028797018963967);
            assert_eq!(result.amount, 100);
            assert!(result.status == TxStatus::Failure);
            assert_eq!(result.description, "\"Record number 1\"");
        }

        #[test]
        fn test_parse_eof() {
            let mut cursor = Cursor::new([]);
            let result = read_record(&mut cursor).unwrap();
            assert!(result.is_none());
        }

        #[test]
        fn test_parse_invalid_magic() {
            let data = [0x00, 0x00, 0x00, 0x00];
            let mut cursor = Cursor::new(data);
            let result = read_record(&mut cursor);
            assert!(matches!(result, Err(ReaderError::InvalidMagic)));
        }
    }
}

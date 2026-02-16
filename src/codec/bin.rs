use super::{Decoder, Encoder};
use crate::codec::bin::parse::read_record;
use crate::codec::bin::write::write_record;
use crate::error::{ReaderError, WriterError};
use crate::transaction::Transaction;
use std::io::{BufReader, Cursor, Read, Write};

pub struct Bin;

const MAGIC: [u8; 4] = [0x59, 0x50, 0x42, 0x4E];
const MIN_RECORD_SIZE: u32 = 8 + 1 + 8 + 8 + 8 + 8 + 1 + 4;
const HEADER_SIZE: usize = 8;

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
        txs.iter().try_for_each(|tx| write_record(w, tx))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Private helpers
// ─────────────────────────────────────────────────────────────────────────────
mod parse {
    use crate::codec::bin::{MAGIC, MIN_RECORD_SIZE};
    use crate::transaction::{TxStatus, TxType};
    use crate::{ReaderError, Transaction};
    use std::io::{ErrorKind, Read};

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
            reason: format!("TxType should be 0 = DEPOSIT, 1 = TRANSFER, 2 = WITHDRAWAL. Received value - {}",
                            tx_type_raw),
        })?;

        let from_user_id = read_be!(r, u64);
        let to_user_id = read_be!(r, u64);
        let amount = read_be!(r, u64);
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
        let real_size = desc_len.checked_add(MIN_RECORD_SIZE).ok_or_else(|| {
            ReaderError::InvalidBinaryFormat {
                reason: "record size overflow".to_string(),
            }
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
            let data = [
                0x59, 0x50, 0x42, 0x4e, // magic = "YPBN"
                0x00, 0x00, 0x00, 0x3f, // record_size = 63
                0x00, 0x03, 0x8d, 0x7e, 0xa4, 0xc6, 0x80, 0x00, // tx_id
                0x00, // tx_type
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // from_user_id
                0x00, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // to_user_id
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, // amount = 100
                0x00, 0x00, 0x01, 0x7c, 0x38, 0x94, 0xfa, 0x60, // timestamp
                0x01, // status
                0x00, 0x00, 0x00, 0x11, // desc_len = 17
                // description string (17 bytes)
                0x22, 0x52, 0x65, 0x63, 0x6f, 0x72, 0x64, 0x20, 0x6e, 0x75, 0x6d, 0x62, 0x65, 0x72,
                0x20, 0x31, 0x22,
            ];

            let mut cursor = Cursor::new(data);
            let result = read_record(&mut cursor).unwrap().unwrap();

            assert_eq!(result.tx_id, 1000000000000000);
            assert_eq!(result.from_user_id, 0);
            assert_eq!(result.to_user_id, 36028797018963967);
            assert_eq!(result.amount, 100);
            assert_eq!(result.status, TxStatus::Failure);
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

mod write {
    use super::*;
    use crate::WriterError;
    use std::io::Write;

    pub(super) fn write_record<W: Write>(w: &mut W, tx: &Transaction) -> Result<(), WriterError> {
        let desc = tx.description.as_bytes();
        let desc_len = desc.len() as u32;
        let size = MIN_RECORD_SIZE + desc_len;
        let mut buf = Vec::with_capacity(HEADER_SIZE+size as usize);

        macro_rules! push_be {
            ($val:expr) => {
                buf.extend_from_slice(&$val.to_be_bytes())
            };
        }

        buf.extend_from_slice(&MAGIC);
        push_be!(size);
        push_be!(tx.tx_id);
        buf.push(tx.tx_type.to_byte());
        push_be!(tx.from_user_id);
        push_be!(tx.to_user_id);
        push_be!(tx.amount);
        push_be!(tx.timestamp);
        buf.push(tx.status.to_byte());
        push_be!(desc_len);
        buf.extend_from_slice(desc);

        w.write_all(&buf).map_err(WriterError::Io)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{TxStatus, TxType};
    use std::io::Cursor;

    // ─────────────────────────────────────────────────────────────────────────
    // Helpers
    // ─────────────────────────────────────────────────────────────────────────

    fn sample_tx() -> Transaction {
        Transaction {
            tx_id: 12345,
            tx_type: TxType::Transfer,
            from_user_id: 100,
            to_user_id: 200,
            amount: 5000,
            timestamp: 1700000000000,
            status: TxStatus::Success,
            description: "Test transaction".to_string(),
        }
    }

    fn make_record(tx: &Transaction) -> Vec<u8> {
        let mut buf = Vec::new();
        write_record(&mut buf, tx).unwrap();
        buf
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Round-trip tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn roundtrip_single() {
        let tx = sample_tx();
        let bytes = make_record(&tx);

        let mut cursor = Cursor::new(bytes);
        let parsed = read_record(&mut cursor).unwrap().unwrap();

        assert_eq!(parsed.tx_id, tx.tx_id);
        assert_eq!(parsed.tx_type, tx.tx_type);
        assert_eq!(parsed.from_user_id, tx.from_user_id);
        assert_eq!(parsed.to_user_id, tx.to_user_id);
        assert_eq!(parsed.amount, tx.amount);
        assert_eq!(parsed.timestamp, tx.timestamp);
        assert_eq!(parsed.status, tx.status);
        assert_eq!(parsed.description, tx.description);
    }

    #[test]
    fn roundtrip_multiple() {
        let txs = vec![
            sample_tx(),
            Transaction {
                tx_id: 999,
                ..sample_tx()
            },
            Transaction {
                description: "Another one".into(),
                ..sample_tx()
            },
        ];

        let mut buf = Vec::new();
        for tx in &txs {
            write_record(&mut buf, tx).unwrap();
        }

        let mut cursor = Cursor::new(buf);
        let mut parsed = Vec::new();
        while let Some(tx) = parse::read_record(&mut cursor).unwrap() {
            parsed.push(tx);
        }

        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0].tx_id, txs[0].tx_id);
        assert_eq!(parsed[1].tx_id, 999);
        assert_eq!(parsed[2].description, "Another one");
    }

    #[test]
    fn roundtrip_empty_description() {
        let tx = Transaction {
            description: "".to_string(),
            ..sample_tx()
        };

        let bytes = make_record(&tx);
        let mut cursor = Cursor::new(bytes);
        let parsed = read_record(&mut cursor).unwrap().unwrap();

        assert_eq!(parsed.description, "");
    }

    #[test]
    fn roundtrip_all_tx_types() {
        for tx_type in [TxType::Deposit, TxType::Transfer, TxType::Withdrawal] {
            let tx = Transaction {
                tx_type,
                ..sample_tx()
            };
            let bytes = make_record(&tx);

            let mut cursor = Cursor::new(bytes);
            let parsed = read_record(&mut cursor).unwrap().unwrap();

            assert_eq!(parsed.tx_type, tx_type);
        }
    }

    #[test]
    fn roundtrip_all_statuses() {
        for status in [TxStatus::Success, TxStatus::Failure, TxStatus::Pending] {
            let tx = Transaction {
                status,
                ..sample_tx()
            };
            let bytes = make_record(&tx);

            let mut cursor = Cursor::new(bytes);
            let parsed = read_record(&mut cursor).unwrap().unwrap();

            assert_eq!(parsed.status, status);
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Parse error tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn parse_eof_returns_none() {
        let mut cursor = Cursor::new([]);
        assert!(parse::read_record(&mut cursor).unwrap().is_none());
    }

    #[test]
    fn parse_invalid_magic() {
        let data = [0xDE, 0xAD, 0xBE, 0xEF];
        let mut cursor = Cursor::new(data);

        assert!(matches!(
            read_record(&mut cursor),
            Err(ReaderError::InvalidMagic)
        ));
    }

    #[test]
    fn parse_truncated_header() {
        let data = MAGIC;
        let mut cursor = Cursor::new(data);

        assert!(matches!(
            parse::read_record(&mut cursor),
            Err(ReaderError::Io(_))
        ));
    }

    #[test]
    fn parse_truncated_body() {
        let mut data = Vec::new();
        data.extend_from_slice(&MAGIC);
        data.extend_from_slice(&50u32.to_be_bytes()); // record_size
        data.extend_from_slice(&[0u8; 10]); // неполные данные

        let mut cursor = Cursor::new(data);

        assert!(matches!(read_record(&mut cursor), Err(ReaderError::Io(_))));
    }

    #[test]
    fn parse_record_size_too_small() {
        let mut data = Vec::new();
        data.extend_from_slice(&MAGIC);
        data.extend_from_slice(&10u32.to_be_bytes()); // меньше MIN_RECORD_SIZE

        let mut cursor = Cursor::new(data);

        assert!(matches!(
            read_record(&mut cursor),
            Err(ReaderError::InvalidBinaryFormat { .. })
        ));
    }

    #[test]
    fn parse_invalid_tx_type() {
        let tx = sample_tx();
        let mut data = make_record(&tx);

        data[16] = 0xFF;

        let mut cursor = Cursor::new(data);

        assert!(matches!(
            read_record(&mut cursor),
            Err(ReaderError::InvalidBinaryFormat { .. })
        ));
    }

    #[test]
    fn parse_invalid_status() {
        let tx = sample_tx();
        let mut data = make_record(&tx);

        // status на позиции: 16 + 1 + 8 + 8 + 8 + 8 = 49
        data[49] = 0xFF;

        let mut cursor = Cursor::new(data);

        assert!(matches!(
            read_record(&mut cursor),
            Err(ReaderError::InvalidBinaryFormat { .. })
        ));
    }

    #[test]
    fn parse_size_mismatch() {
        let tx = sample_tx();
        let mut data = make_record(&tx);

        let wrong_size = 100u32.to_be_bytes();
        data[4..8].copy_from_slice(&wrong_size);

        let mut cursor = Cursor::new(data);

        assert!(matches!(
            read_record(&mut cursor),
            Err(ReaderError::InvalidBinaryFormat { .. })
        ));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Write tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn write_produces_valid_magic() {
        let tx = sample_tx();
        let data = make_record(&tx);

        assert_eq!(&data[0..4], &MAGIC);
    }

    #[test]
    fn write_correct_record_size() {
        let tx = Transaction {
            description: "Hello".to_string(), // 5 байт
            ..sample_tx()
        };
        let data = make_record(&tx);

        let size = u32::from_be_bytes(data[4..8].try_into().unwrap());
        assert_eq!(size, MIN_RECORD_SIZE + 5);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Decoder/Encoder integration tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn decoder_empty_input() {
        let bin = Bin;
        let mut cursor = Cursor::new([]);

        let txs = bin.decode(&mut cursor).unwrap();
        assert!(txs.is_empty());
    }

    #[test]
    fn encoder_decoder_roundtrip() {
        let bin = Bin;
        let txs = vec![
            sample_tx(),
            Transaction {
                tx_id: 777,
                ..sample_tx()
            },
        ];

        let mut buf = Vec::new();
        bin.encode(&txs, &mut buf).unwrap();

        let mut cursor = Cursor::new(buf);
        let decoded = bin.decode(&mut cursor).unwrap();

        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded[0].tx_id, txs[0].tx_id);
        assert_eq!(decoded[1].tx_id, 777);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Edge cases
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn max_values() {
        let tx = Transaction {
            tx_id: u64::MAX,
            tx_type: TxType::Withdrawal,
            from_user_id: u64::MAX,
            to_user_id: u64::MAX,
            amount: u64::MAX,
            timestamp: u64::MAX,
            status: TxStatus::Pending,
            description: "Max values test".to_string(),
        };

        let bytes = make_record(&tx);
        let mut cursor = Cursor::new(bytes);
        let parsed = read_record(&mut cursor).unwrap().unwrap();

        assert_eq!(parsed.tx_id, u64::MAX);
        assert_eq!(parsed.amount, u64::MAX);
    }

    #[test]
    fn unicode_description() {
        let tx = Transaction {
            description: "Привет 🦀 мир!".to_string(),
            ..sample_tx()
        };

        let bytes = make_record(&tx);
        let mut cursor = Cursor::new(bytes);
        let parsed = read_record(&mut cursor).unwrap().unwrap();

        assert_eq!(parsed.description, "Привет 🦀 мир!");
    }
}

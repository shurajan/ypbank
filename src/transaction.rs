use crate::error::{ReaderError, WriterError};

#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: u64,
}

pub trait TransactionDecoder {
    fn decode_all<R: std::io::Read>(&self, r: &mut R) -> Result<Vec<Transaction>, ReaderError>;
}

pub trait TransactionEncoder {
    fn encode_all<W: std::io::Write>(&self, w: &mut W) -> Result<(), WriterError>;
}

pub fn read_transactions<T: TransactionDecoder, R: std::io::Read>(
    decoder: &mut T,
    r: &mut R,
) -> Result<Vec<Transaction>, ReaderError> {
    decoder.decode_all(r)
}

pub fn write_transactions<T: TransactionEncoder, W: std::io::Write>(
    encoder: &mut T,
    r: &mut W,
) -> Result<(), WriterError> {
    encoder.encode_all(r)
}

use crate::error::{ReaderError, WriterError};
use crate::transaction::{Transaction, TransactionDecoder, TransactionEncoder};
use std::io::Read;

pub struct Csv;

impl TransactionDecoder for Csv {
    fn decode_all<R: Read>(&self, r: &mut R) -> Result<Vec<Transaction>, ReaderError> {
        todo!()
    }
}

impl TransactionEncoder for Csv {
    fn encode_all<W: std::io::Write>(&self, w: &mut W) -> Result<(), WriterError> {
        todo!()
    }
}

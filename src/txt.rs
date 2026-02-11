use crate::error::{ReaderError, WriterError};
use crate::transaction::{Transaction, TransactionDecoder, TransactionEncoder};
use std::io::{Read, Write};

pub struct Txt;

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

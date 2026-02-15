use super::{Decoder, Encoder};
use crate::error::{ReaderError, WriterError};
use crate::transaction::Transaction;
use std::io::{Read, Write};

pub struct Bin;

impl Decoder for Bin {
    fn decode<R: Read>(&self, r: &mut R) -> Result<Vec<Transaction>, ReaderError> {
        todo!()
    }
}

impl Encoder for Bin {
    fn encode<W: Write>(&self, txs: &[Transaction], w: &mut W) -> Result<(), WriterError> {
        todo!()
    }
}

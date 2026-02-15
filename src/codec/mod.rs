mod bin;
mod csv;
mod txt;

pub use bin::Bin;
pub use csv::Csv;
pub use txt::Txt;

use crate::error::{ReaderError, WriterError};
use crate::transaction::Transaction;
use std::io::{Read, Write};

pub trait Decoder {
    fn decode<R: Read>(&self, r: &mut R) -> Result<Vec<Transaction>, ReaderError>;
}

pub trait Encoder {
    fn encode<W: Write>(&self, txs: &[Transaction], w: &mut W) -> Result<(), WriterError>;
}

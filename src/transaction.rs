use crate::error::{ReaderError, WriterError};
use std::io::{Cursor, Read, Write};

#[derive(Debug, Clone)]
pub struct Transaction {
    pub tx_id: u64,
    pub tx_type: TxType,
    pub from_user_id: u64,
    pub to_user_id: u64,
    pub amount: u64,
    pub timestamp: u64,
    pub status: TxStatus,
    pub description: String,
}

#[derive(Debug, Clone)]
pub enum TxType {
    Deposit,
    Transfer,
    Withdrawal,
}

#[derive(Debug, Clone)]
pub enum TxStatus {
    Success,
    Failure,
    Pending,
}

pub trait TransactionDecoder {
    fn decode_all<R: Read>(&self, r: &mut R) -> Result<Vec<Transaction>, ReaderError>;
}

pub trait TransactionEncoder {
    fn encode_all<W: Write>(&self, txs: &Vec<Transaction>, w: &mut W) -> Result<(), WriterError>;
}

pub fn read_transactions<R: Read>(
    decoder: &impl TransactionDecoder,
    r: &mut R,
) -> Result<Vec<Transaction>, ReaderError> {
    decoder.decode_all(r)
}

pub fn write_transactions<W: Write>(
    encoder: &impl TransactionEncoder,
    txs: &Vec<Transaction>,
    w: &mut W,
) -> Result<(), WriterError> {
    encoder.encode_all(txs, w)
}

mod bin;
mod csv;
mod txt;

pub use bin::Bin;
pub use csv::Csv;
pub use txt::Txt;

use crate::error::{ReaderError, WriterError};
use crate::transaction::Transaction;
use std::io::{Read, Write};

/// A transaction decoder.
///
/// Implementors of this trait can parse a stream of bytes
/// and produce a list of [`Transaction`] values.
///
/// Decoders are typically format-specific (e.g. [`Csv`], [`Bin`]).
///
/// ## Input
///
/// The provided reader must contain transaction data in the
/// expected format.
///
/// ## Output
///
/// On success, returns a vector of decoded transactions.
///
/// ## Errors
///
/// Returns [`ReaderError`] if the input is malformed or cannot
/// be parsed into valid transactions.
///
/// ## Example
///
/// ```
/// use parser::{Decoder, Csv};
///
/// let data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
/// 1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,"Initial funding"
/// "#;
/// let txs = Csv.decode(&mut data.as_bytes()).unwrap();
///
/// assert!(!txs.is_empty());
/// ```
pub trait Decoder {
    /// Decodes transactions from the given reader.
    fn decode<R: Read>(&self, r: &mut R) -> Result<Vec<Transaction>, ReaderError>;
}

/// A transaction encoder.
///
/// Implementors of this trait can serialize a slice of
/// [`Transaction`] values into an output stream.
///
/// Encoders are typically format-specific (e.g. [`Csv`], [`Bin`]).
///
/// ## Output
///
/// The encoded representation is written into the provided writer.
///
/// ## Errors
///
/// Returns [`WriterError`] if writing fails or if transactions
/// cannot be represented in the target format.
///
/// ## Example
///
/// ```
/// use std::io::Cursor;
/// use parser::{Encoder, Csv, Transaction, TxStatus, TxType};
///
/// let txs = vec![Transaction {
///    tx_id: 1001,
///    tx_type: TxType::Deposit,
///    from_user_id: 0,
///    to_user_id: 501,
///    amount: 50000,
///    timestamp: 1672531200000,
///    status: TxStatus::Success,
///    description: "\"Test\"".to_string(),
/// }];
///
/// let mut buffer = Cursor::new(Vec::new());
/// let result = Csv.encode(&txs, &mut buffer);
/// assert!(result.is_ok());
/// ```
pub trait Encoder {
    /// Encodes transactions into the given writer.
    fn encode<W: Write>(&self, txs: &[Transaction], w: &mut W) -> Result<(), WriterError>;
}

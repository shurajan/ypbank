//! # ypbank
//!
//! `ypbank` is a transaction codec library that provides tools for
//! encoding and decoding financial transactions in multiple formats.
//!
//! //! ## Design
//!
//! Codecs implement [`Decoder`] and [`Encoder`] and operate on any
//! type implementing [`Read`] or [`Write`], making them compatible
//! with files, network streams, and in-memory buffers.
//!
//! ## Supported Formats
//!
//! - [`Csv`] — CSV encoding/decoding
//! - [`Txt`] — plain text encoding/decoding
//! - [`Bin`] — compact binary encoding/decoding
//!
//! ## Core Traits
//!
//! - [`Decoder`] — decode transactions from a stream
//! - [`Encoder`] — encode transactions into a stream
//!
//! ## Example: Decoding CSV Transactions
//!
//! ```
//! use ypbank::{Decoder, Csv};
//!
//! let data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
//! 1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,"Initial funding"
//! "#;
//!
//! let txs = Csv.decode(&mut data.as_bytes()).unwrap();
//!
//! assert_eq!(txs.len(), 1);
//! assert_eq!(txs[0].tx_id, 1001);
//! ```
//!
//! ## Errors
//!
//! All decoding operations return [`ReaderError`] on invalid input,
//! while encoding operations return [`WriterError`].
#![warn(missing_docs)]

/// Format codecs (CSV/TXT/BIN) and the core [`Decoder`] / [`Encoder`] traits.pub mod codec;
pub mod codec;

/// Error types returned by decoding and encoding operations.
pub mod error;

/// Core data model: [`Transaction`], [`TxType`], and [`TxStatus`].
pub mod transaction;

pub use codec::{Bin, Csv, Txt};
pub use codec::{Decoder, Encoder};
pub use error::{ReaderError, WriterError};
pub use transaction::Transaction;
pub use transaction::TxStatus;
pub use transaction::TxType;

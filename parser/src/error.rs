use std::fmt;

/// An error returned by decoding operations (e.g. [`Csv`], ['Txt'],  [`Bin`]).
///
/// This type describes both I/O failures and format/validation issues.
///
/// ## Notes
///
/// - Many variants include a `line_no` field. Line numbers are **1-based**
///   and typically refer to the **data line** in the input (including the header
///   if your decoder counts it). Document the exact convention in the codec
///   implementation if it matters.
/// - Use [`std::error::Error::source`] (via the `Io` variant) to access the
///   underlying I/O error.
///
/// ## Common cases
///
/// - A malformed CSV header yields [`ReaderError::InvalidCsvHeader`].
/// - A row that cannot be parsed yields [`ReaderError::InvalidRow`].
/// - A field with an unknown/unsupported name yields [`ReaderError::UnknownField`].
/// - Binary input with wrong magic bytes yields [`ReaderError::InvalidMagic`].
#[derive(Debug)]
pub enum ReaderError {
    /// An underlying I/O error occurred while reading from the input stream.
    Io(std::io::Error),

    /// The CSV header row is invalid or does not match the expected schema.
    InvalidCsvHeader {
        /// The header line provided by the input.
        header: String,
    },

    /// A whole row is invalid or can not be parsed.
    InvalidRow {
        /// Line number where the error occurred.
        line_no: usize,
        /// Human-readable explanation of what went wrong.
        reason: String,
    },

    /// A specific field contains an invalid value.
    InvalidFieldValue {
        /// Line number where the error occurred.
        line_no: usize,
        /// Field/column name (e.g., `TX_TYPE`).
        field: String,
        /// The raw value from the input.
        value: String,
    },

    /// One or more required fields are missing in the Transaction, can be thrown by Txt.
    MissingFields {
        /// Line number where the transaction starts.
        line_no: usize,
        /// Names of missing fields/columns.
        fields: Vec<String>,
    },

    /// The input contains an unexpected/unknown field.
    UnknownField {
        /// Line number where the error occurred, can be thrown by Txt.
        line_no: usize,
        /// The unknown field/column name.
        field: String,
    },

    /// A field appears more than once, can be thrown by Txt..
    DuplicateField {
        /// Line number where the transaction starts.
        line_no: usize,
        /// The duplicated field/column name.
        field: String,
    },

    /// Binary input does not start with the expected magic bytes.
    InvalidMagic,

    /// Binary input is structurally invalid.
    /// Use `reason` to provide a human-readable explanation (e.g. "unexpected EOF",
    InvalidBinaryFormat {
        /// Human-readable explanation of the binary format violation.
        reason: String,
    },
}

/// An error returned by encoding operations.
#[derive(Debug)]
pub enum WriterError {
    /// An underlying I/O error occurred while writing to the output stream.
    Io(std::io::Error),
    /// The provided input data is not valid for encoding.
    InvalidData {
        /// Human-readable explanation of what makes the data invalid.
        reason: String,
    },
}
impl fmt::Display for ReaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{}", e),

            Self::InvalidCsvHeader { header } => {
                write!(f, "Invalid CSV header: '{header}'")
            }
            Self::InvalidRow { line_no, reason } => {
                write!(f, "Line {line_no} - invalid row: {reason}")
            }

            Self::InvalidFieldValue {
                line_no,
                field,
                value,
            } => {
                write!(
                    f,
                    "Line {line_no} - invalid field value: {field}, value: {value}"
                )
            }
            Self::MissingFields { line_no, fields } => {
                write!(f, "line {}: missing fields: {}", line_no, fields.join(", "))
            }
            Self::UnknownField { line_no, field } => {
                write!(f, "line {}: unknown field: {}", line_no, field)
            }

            Self::DuplicateField { line_no, field } => {
                write!(f, "line {}: duplicate field: {}", line_no, field)
            }
            Self::InvalidMagic => {
                write!(f, "Invalid Magic")
            }
            Self::InvalidBinaryFormat { reason } => {
                write!(f, "Invalid binary format: {reason}")
            }
        }
    }
}

impl fmt::Display for WriterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WriterError::Io(e) => write!(f, "{}", e),
            WriterError::InvalidData { reason } => {
                write!(f, "Invalid data: {reason}")
            }
        }
    }
}

impl std::error::Error for ReaderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl std::error::Error for WriterError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}
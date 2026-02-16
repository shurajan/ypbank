use std::fmt;

#[derive(Debug)]
pub enum ReaderError {
    Io(std::io::Error),
    InvalidCsvHeader {
        header: String,
    },
    InvalidRow {
        line_no: usize,
        reason: String,
    },
    InvalidFieldValue {
        line_no: usize,
        field: String,
        value: String,
    },

    MissingFields {
        line_no: usize,
        fields: Vec<String>,
    },

    UnknownField {
        line_no: usize,
        field: String,
    },

    DuplicateField {
        line_no: usize,
        field: String,
    },

    InvalidMagic,

    InvalidBinaryFormat {
        reason: String,
    },
}

#[derive(Debug)]
pub enum WriterError {
    Io(std::io::Error),
    InvalidData { reason: String },
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

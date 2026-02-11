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
}

#[derive(Debug)]
pub enum WriterError {
    Io(std::io::Error),
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
        }
    }
}

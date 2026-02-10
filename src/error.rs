use std::fmt;
use std::fmt::{write, Formatter};


#[derive(Debug)]
pub enum ReaderError {
    InvalidFormat,
    InvalidDataFormat{ field_name: String, line: usize},
    Io(std::io::Error),
}

#[derive(Debug)]
pub enum WriterError {
    Io(std::io::Error),
}


impl fmt::Display for ReaderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ReaderError::InvalidFormat => {
                write!(f, "Некорректный формат CSV (заголовок)")
            }

            ReaderError::InvalidDataFormat { field_name, line } => {
                write!(
                    f,
                    "Некорректные данные в поле '{}', строке {}",
                    field_name, line
                )
            }

            ReaderError::Io(err) => {
                write!(f, "Ошибка ввода-вывода: {}", err)
            }
        }
    }
}
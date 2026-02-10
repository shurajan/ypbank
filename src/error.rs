#[derive(Debug)]
pub enum ReaderError {
    InvalidFormat,
    Io(std::io::Error),
}

#[derive(Debug)]
pub enum WriterError {
    Io(std::io::Error),
}

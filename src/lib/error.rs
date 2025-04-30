pub enum Error {
    BadCommand(String),
    Operation(String),
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::BadCommand(s) => write!(f, "Bad command: {}", s),
            Error::Operation(s) => write!(f, "Operation error: {}", s),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::BadCommand(s) => write!(f, "Bad command: {}", s),
            Error::Operation(s) => write!(f, "Operation error: {}", s),
        }
    }
}

use std::{error::Error, fmt};

#[derive(Debug)]
pub enum RollerError {
    EvalError(String),
    ParserError(String),
    OtherError,
}

impl fmt::Display for RollerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RollerError::EvalError(msg) | RollerError::ParserError(msg) => write!(f, "{}", msg),
            RollerError::OtherError => write!(f, "an unknown error"),
        }
    }
}

impl Error for RollerError {}

impl From<()> for RollerError {
    fn from(_value: ()) -> Self {
        RollerError::OtherError
    }
}

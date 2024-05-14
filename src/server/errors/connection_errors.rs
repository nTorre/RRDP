use std::fmt::{Display, Formatter};

/// # Config Errors
/// Defining errors thrown loading the configs from the file
/// ### Errors
/// - **IOError**: error while loading config file
///
#[derive(Debug)]
pub enum ConnectionError {
    RunTimeError(),
    CreateConnectionError(String),
    AddressParsingError(String),
    OpenBiError(String),
    SendingError(String)
}

impl Display for ConnectionError{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            ConnectionError::RunTimeError() => {
                write!(f, "Error while creating rrdp server runtime")
            }
            ConnectionError::CreateConnectionError(ref desc) => {
                write!(f, "Error while creating connection with client. {}", desc)
            }
            ConnectionError::AddressParsingError(ref address) => {
                write!(f, "Error while parsing the address {}", address)
            }
            ConnectionError::OpenBiError(ref desc) => {
                write!(f, "Error while opening bidirectional channel. {}", desc)
            }
            ConnectionError::SendingError(ref desc) => {
                write!(f, "Error while sending message. {}", desc)
            }
        }
    }
}


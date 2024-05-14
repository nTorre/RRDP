use std::fmt::{Display, Formatter};

/// # Crypto Errors
/// Defining errors thrown by a cryptography error
/// or every error during the very first phase of connection
/// ### Errors
/// - **IOError**: if the path of the cert specified is not found
/// - **PCKS8ReadError**: cannot read PCKS#8 private keys in file due to different causes
/// - **PCKS8NotFound**: can read the file, but not PCKS#8 private keys found
/// - **CertsReadError**: cannot read certs from file
/// - **ServerConfigError**: error while creating quinn server config
/// - **GenerationError**: error while generating certs
#[derive(Debug)]
pub enum CryptoError {
    IOError(String),
    PCKS8ReadError(String),
    PCKS8NotFound(String),
    CertsReadError(String),
    ServerConfigError(String),
    GenerationError(String)
}

impl Display for CryptoError{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            CryptoError::IOError(ref path) => {
                write!(f, "IO error. File {:?} not found", path)
            }
            CryptoError::PCKS8ReadError(ref path) => {
                write!(f, "Cannot read PKCS#8 private keys in file {:?}", path)
            }
            CryptoError::PCKS8NotFound(ref path) => {
                write!(f, "No PKCS#8 keys found in file {:?}", path)
            }
            CryptoError::CertsReadError(ref path) => {
                write!(f, "Cannot read certs from file {:?}", path)
            }
            CryptoError::ServerConfigError(ref desc) => {
                write!(f, "Error while creating quinn server config: {:?}", desc)
            }
            CryptoError::GenerationError(ref desc) => {
                write!(f, "Error while generating certs: {:?}", desc)
            }
        }
    }
}


use std::fmt::{Display, Formatter};

/// # Config Errors
/// Defining errors thrown loading the configs from the file
/// ### Errors
/// - **IOError**: error while loading config file
///
#[derive(Debug)]
pub enum ConfigError {
    IOError(String),
    TOMLNotParsed(String),
    ParamsMissing(String),
    DesktopEnvError(String),
    ScreenDimensionsError(String),
    ConversionError(String)
}

impl Display for ConfigError{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            ConfigError::IOError(ref path) => {
                write!(f, "IO error while loading rrdp config. File {} not found", path)
            }
            ConfigError::TOMLNotParsed(ref path) => {
                write!(f, "File {} not well formatted. See documentation", path)
            }
            ConfigError::ParamsMissing(ref desc) => {
                write!(f, "{}", desc)
            }
            ConfigError::DesktopEnvError(ref desc) => {
                write!(f, "{} env not known. See documentation", desc)
            }
            ConfigError::ScreenDimensionsError(ref desc) => {
                write!(f, "{}", desc)
            }
            ConfigError::ConversionError(ref desc) => {
                write!(f, "{}", desc)
            }
        }
    }
}


use std::fmt::{Display, Formatter};
use std::str::FromStr;
use crate::configs::setup_config;
use crate::errors::config_errors::ConfigError;
use crate::display::ScreenDimensions;

#[derive(Debug)]
pub enum DesktopEnv{
    Xfce4,
    OpenBox
}

/// # ServerConfig
/// General server information loaded on the start of the program
#[derive(Debug)]
pub struct ServerConfig {
    pub connection: ConnectionConfig,
    pub display: DisplayConfig,
    pub settings: SettingsConfig
}

#[derive(Debug)]
pub struct ConnectionConfig {
    pub generate_certs: bool,
    pub key_path: String,
    pub certs_path: String
}

#[derive(Debug)]
pub struct DisplayConfig {
    pub custom_display: bool,
    pub display: String,
    pub desktop_env: DesktopEnv,
    pub dimensions: ScreenDimensions,
    pub max_slice_size: i32
}

#[derive(Debug)]
pub struct SettingsConfig {
    pub log_path: String,
}

impl ServerConfig{
    pub fn from_file(path: String)->Result<ServerConfig, ConfigError>{
        setup_config(Some(path))
    }
}

/// # Defaults for Config
impl Default for SettingsConfig{
    fn default() -> Self {
        SettingsConfig{
            log_path: "/var/log/rrdp.log".to_string()
        }
    }
}

impl Default for DisplayConfig{
    fn default() -> Self {
        DisplayConfig{
            custom_display: true,
            display: ":1".to_string(),
            desktop_env: DesktopEnv::Xfce4,
            dimensions: (1024, 768, 24),
            max_slice_size: 0,
        }
    }
}

impl Default for ConnectionConfig{
    fn default() -> Self {
        ConnectionConfig{
            generate_certs: true,
            key_path: "certs/key.pem".to_string(),
            certs_path: "certs/cert.pem".to_string(),
        }
    }
}

impl Default for ServerConfig{
    fn default() -> Self {
        ServerConfig{
            connection: Default::default(),
            display: Default::default(),
            settings: Default::default(),
        }
    }
}

impl FromStr for DesktopEnv{
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "xfce4" => Ok(DesktopEnv::Xfce4),
            "openbox" => Ok(DesktopEnv::OpenBox),
            env => Err(ConfigError::DesktopEnvError(env.to_string()))
        }
    }
}

impl Display for SettingsConfig{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "not implemented")
    }
}

impl Display for DisplayConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "custom_display = {}\n", self.custom_display).unwrap();
        write!(f, "display = {}\n", self.display).unwrap();
        write!(f, "desktop_env = {:?}\n", self.desktop_env).unwrap();
        write!(f, "dimensions = {:?}\n", self.dimensions).unwrap();
        write!(f, "max_slice_size = {}\n", self.max_slice_size).unwrap();

        Ok(())
    }
}

impl Display for ConnectionConfig{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "generate_certs = {}\n", self.generate_certs).unwrap();
        write!(f, "key_path = {}\n", self.key_path).unwrap();
        write!(f, "certs_path = {}\n", self.certs_path).unwrap();

        Ok(())
    }
}

impl Display for ServerConfig{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "---------------------------\n").unwrap();
        write!(f, "[connection]\n{}\n[display]\n{}\n[settings]\n{}",
               self.connection, self.display, self.settings).unwrap();
        write!(f, "\n---------------------------").unwrap();

        Ok(())
    }
}
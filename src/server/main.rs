mod handlers;
mod cryptography;
mod display;
mod sesman;
mod errors;
mod configs;
mod rd;

use std::fs;
use handlers::handle_connection;
use std::sync::Arc;
use log::{error, info, warn};
use quinn::{Endpoint, ServerConfig as QuinnServerConfig};
use tokio::runtime::Runtime;
use toml::Table;
use crate::configs::{setup_config, ServerConfig};
use crate::cryptography::config_server_security;
use crate::errors::config_errors::ConfigError;
use crate::errors::connection_errors::ConnectionError;
use crate::errors::crypto_errors::CryptoError;
use crate::rd::start_listening;

fn main(){

    // setup log
    errors::setup_log::setup_logging().expect("Failed to initialize logging.");

    // setup config
    let config = ServerConfig::from_file("rrdp.config".to_string())
        .unwrap_or_else(|err| {
        warn!("{}", err);
        ServerConfig::default()
    });
    info!("Configs:\n{}", config);

    // setup cryptography
    let mut server_config;
    if config.connection.generate_certs {
        server_config = config_server_security(None);
    } else {
        server_config = config_server_security(
            Some((config.connection.certs_path.as_str(), config.connection.key_path.as_str())));
    }

    let server_crypto = match server_config{
        Ok(server_config) => server_config,
        Err(err) => {
            error!("{}", err);
            panic!()
        }
    };


    // starting server
    match start_listening(server_crypto){
        Ok(_) => {},
        Err(err) => {
            error!("{}", err);
            panic!()
        }
    }

}
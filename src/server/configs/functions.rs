use std::fs;
use std::str::FromStr;
use log::{error, info, warn};
use crate::errors::config_errors::ConfigError;
use toml::{Table, Value};
use crate::configs::{ConnectionConfig, DesktopEnv, DisplayConfig, ServerConfig};
use crate::display::ScreenDimensions;

pub fn setup_config(path: Option<String>) -> Result<ServerConfig, ConfigError>{

    if path.is_none(){
        return Ok(ServerConfig::default());
    }

    let path = path.unwrap();
    info!("Parsing {:?}", path);
    let toml_str = match fs::read_to_string(&path){
        Ok(toml_str) => toml_str,
        Err(_) => {
            return Err(ConfigError::IOError(path));
        }
    };

    let conf = match toml_str.parse::<Table>(){
        Ok(conf) => conf,
        Err(_) => {
            return Err(ConfigError::TOMLNotParsed(path))
        }
    };

    let connection = if conf.contains_key("connection"){
        &conf["connection"]
    } else{
        return Err(ConfigError::ParamsMissing(
            "Config file is not complete, missing connection section.".to_string()));
    };

    let display = if conf.contains_key("display"){
        &conf["display"]
    } else{
        return Err(ConfigError::ParamsMissing(
            "Config file is not complete, missing display section.".to_string()));
    };

    let _settings = if conf.contains_key("settings"){
        &conf["settings"]
    } else{
        return Err(ConfigError::ParamsMissing(
            "Config file is not complete, missing settings section.".to_string()));

    };

    let connection_config = match check_connection(connection){
        Ok(connection_config) => connection_config,
        Err(err) => {
            return Err(err);
        }
    };
    let display_config = match check_display(display){
        Ok(display_config) => display_config,
        Err(err) => {
            return Err(err);
        }
    };

    let server_config = ServerConfig{
        connection: connection_config,
        display: display_config,
        settings: Default::default(),
    };

    Ok(server_config)
}


fn check_connection(connection: &Value)->Result<ConnectionConfig, ConfigError>{
    let mut connection_config = ConnectionConfig::default();

    let generate_certs = connection.get("generate_certs");
    if generate_certs.is_none() {
        return Err(ConfigError::ParamsMissing("Missing 'generate_certs' field".to_string()));
    } else{
        connection_config.generate_certs =
            generate_certs.unwrap().as_bool().unwrap();
    }

    if !connection_config.generate_certs{
        let key_path = connection.get("key_path");
        if key_path.is_none() {
            return Err(ConfigError::ParamsMissing("Key path is missing! \
        If you want to use your custom certs, specify path. \
        Using default config file".to_string()));
        } else {
            connection_config.key_path =
                key_path.unwrap().as_str().unwrap().to_string();
        }

        let certs_path = connection.get("certs_path");
        if certs_path.is_none() {
            return Err(ConfigError::ParamsMissing("Certs path is missing! \
        If you want to use your custom certs, specify path. \
        Using default config file".to_string()));
        } else {
            connection_config.certs_path =
                certs_path.unwrap().as_str().unwrap().to_string();
        }
    }


    Ok(connection_config)
}

fn check_display(display: &Value)->Result<DisplayConfig, ConfigError>{
    let mut display_config = DisplayConfig::default();

    let custom_display = display.get("custom_display");
    if custom_display.is_none(){
        return Err(ConfigError::ParamsMissing("Missing 'custom_display' field".to_string()));
    } else {
        display_config.custom_display =
            custom_display.unwrap().as_bool().unwrap();
    }

    let display_number = display.get("display");
    if display_number.is_none() {
        return Err(ConfigError::ParamsMissing("Missing 'display' field".to_string()));
    } else {
        display_config.display =
            display_number.unwrap().as_str().unwrap().to_string();
    }

    let desktop_env = display.get("desktop_env");
    if desktop_env.is_none() {
        return Err(ConfigError::ParamsMissing("Missing 'desktop_env' field".to_string()));
    } else {
        match DesktopEnv::from_str(desktop_env.unwrap().as_str().unwrap()){
            Ok(env) => { display_config.desktop_env = env}
            Err(err) => {
                return Err(ConfigError::DesktopEnvError(
                    format!("{:?}. Using default config [{:?}]",
                            err.to_string(), DisplayConfig::default().desktop_env)))
            }
        };
    }

    let dimensions = display.get("dimensions");
    if dimensions.is_none() {
        return Err(ConfigError::ParamsMissing("Missing 'dimensions' field".to_string()));
    } else {

        let vec_dimensions: Vec<&str> =
            dimensions.unwrap().as_str().unwrap().split("x").map(str::trim).collect();

        if vec_dimensions.len() != 3 {
            return Err(
                ConfigError::ScreenDimensionsError(
                    "'dimensions' field has not 3 values".to_string()));
        } else {
            let mut vec_num_dimensions: Vec<i32> = vec![];
            for dimension in vec_dimensions{
                match dimension.parse::<i32>() {
                    Ok(n) => vec_num_dimensions.push(n),
                    Err(_) => {
                        return Err(ConfigError::ScreenDimensionsError(
                            "Error while parsing 'dimensions'".to_string()));
                    },
                }
            }
            display_config.dimensions = (vec_num_dimensions[0],
                                         vec_num_dimensions[1],
                                         vec_num_dimensions[2]);
        }
    }

    let max_slice_size = display.get("max_slice_size");
    if max_slice_size.is_none() {
        return Err(ConfigError::ParamsMissing("Missing 'max_slice_size' field".to_string()));
    } else {
        match max_slice_size.unwrap().as_integer(){
            None => {
                return Err(ConfigError::ConversionError("Field 'max_slice_size' not a number".to_string()));
            }
            Some(n) => {
                display_config.max_slice_size = n as i32;
            }
        };
    }
    Ok(display_config)
}
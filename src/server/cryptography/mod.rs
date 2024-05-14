use std::fs::File;
use std::io::BufReader;
use log::info;
use crate::errors::crypto_errors::CryptoError;
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::certs;
use rcgen::{CertifiedKey, Error, generate_simple_self_signed};


/// # Load Certs
/// Load certs from a specific path
///
/// **Args:**
/// - path: &str, the path of the cert
fn load_certs(path: &str) -> Result<Vec<Certificate>, CryptoError> {
    info!("Loading cert from path {:?}", path);
    let certfile = match File::open(path){
        Ok(certfile) => certfile,
        Err(_) => return Err(
            CryptoError::IOError(
                String::from(path)))
    };
    let mut reader = BufReader::new(certfile);

    let certs = match certs(&mut reader){
        Ok(certs) => certs,
        Err(_) => return Err(
            CryptoError::CertsReadError(
                String::from(path)))
    };

    let certs = certs
        .into_iter()
        .map(Certificate)
        .collect();

    Ok(certs)
}

/// # Load Key
/// Load key from a specific path
///
/// **Args:**
/// - path: &str, the path of the key
fn load_keys(path: &str) -> Result<PrivateKey, CryptoError> {
    info!("Loading key from path {:?}", path);
    let keyfile = match File::open(path){
        Ok(keyfile) => keyfile,
        Err(_) => return Err(
            CryptoError::IOError(
                String::from(path)))
    };

    // Tenta di leggere le chiavi PKCS#8.
    let mut reader = BufReader::new(keyfile);
    let keys = match rustls_pemfile::pkcs8_private_keys(&mut reader){
        Ok(keys) => keys,
        Err(_) =>  return Err(
            CryptoError::PCKS8ReadError(
                String::from(path)))
    };

    // Controlla se è stata trovata almeno una chiave.
    if let Some(key) = keys.into_iter().next() {
        Ok(PrivateKey(key))
    } else {
        Err(CryptoError::PCKS8NotFound(String::from(path)))
    }
}

/// # Generate Certificates
/// A function which generate auto signed certificates
///
fn create_certs()
    ->(Result<(Vec<Certificate>, PrivateKey), CryptoError>){

    info!("Generating certificates");
    let subject_alt_names = vec!["hello.world.example".to_string(),
                                 "localhost".to_string()];

    let (cert, key_pair) = match generate_simple_self_signed(subject_alt_names){
        Ok(res) => (res.cert, res.key_pair),
        Err(e) => return Err(CryptoError::GenerationError(e.to_string()))
    };

    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();

    let binding = pem::parse(cert_pem).unwrap();
    let cert_der = binding.contents();

    let binding = pem::parse(key_pem).unwrap();
    let key_der = binding.contents();

    let cert = Certificate(Vec::from(cert_der));
    let key = PrivateKey(Vec::from(key_der));
    Ok((vec![cert], key))
}

/// Configures the server's security settings using TLS certificates and a private key.
///
/// This function initializes the `ServerConfig` for a server, setting up its TLS configuration
/// based on provided certificates and a private key, or generates new ones if not provided.
///
/// # Arguments
///
/// * `certs` - An optional tuple containing file paths to the TLS certificate and the private key.
///     If `None`, new self-signed certificate and key are generated automatically.
///
/// # Returns
///
/// Returns a `Result` containing `ServerConfig` on success, or `CryptoError` on failure.
///
/// # Errors
///
/// This function can return a `CryptoError` in several cases:
/// - `ServerConfigError` if there is a problem setting up the TLS configuration.
/// - Errors from `create_certs`, `load_certs`, or `load_keys` if there is a problem reading from files
///   or generating the certificates.
///
/// # Examples
///
/// ## Using custom certificate and key files:
///
/// ```rust
/// let config = config_server_security(Some(("/path/to/certificate.pem", "/path/to/key.pem")));
/// match config {
///     Ok(server_config) => println!("Server configured successfully."),
///     Err(e) => println!("Failed to configure server: {:?}", e),
/// }
/// ```
///
/// ## Auto-generating certificate and key:
///
/// ```rust
/// let config = config_server_security(None);
/// match config {
///     Ok(server_config) => println!("Server configured successfully with generated credentials."),
///     Err(e) => println!("Failed to configure server: {:?}", e),
/// }
/// ```
///
/// # Note
///
/// The function uses `ServerConfig::builder` to create a default configuration with no client
/// authentication and safe defaults. This setup assumes the server does not require client certificates
/// for TLS connections, which is suitable for server-centric authentication and encryption setups.
pub fn config_server_security(certs: Option<(&str, &str)>)
    -> Result<ServerConfig, CryptoError> {

    let (certs, key) = match certs {
        Some((cert_path, key_path)) => {
            (load_certs(cert_path), load_keys(key_path))
        }
        None => {
            let res = create_certs();
            match res{
                Ok(res) => {
                    (Ok(res.0), Ok(res.1))
                }
                Err(e) => {
                    return Err(e)
                }
            }
        }
    };

    let certs = match certs{
        Ok(certs) => certs,
        Err(err) => return Err(err)
    };

    let key = match key {
        Ok(key) => key,
        Err(err) => return Err(err)
    };

    let server_crypto = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key);

    let server_crypto = match server_crypto {
        Ok(server_crypto) => server_crypto,
        Err(e) => return Err(
            CryptoError::ServerConfigError(
                e.to_string()))
    };

    return Ok(server_crypto);
}
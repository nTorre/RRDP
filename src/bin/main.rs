use rustls_pemfile::rsa_private_keys;
use anyhow::{Context, Result};
use quinn::{Connection, Endpoint, ServerConfig};
use rustls::{Certificate, PrivateKey};
use std::{fs::File, io::BufReader, sync::Arc};
use std::time::Duration;
use image::DynamicImage;
use rustls_pemfile::certs;
use tokio::runtime::Runtime;
use tokio::time;
use image::io::Reader as ImageReader;

fn load_certs(path: &str) -> Result<Vec<Certificate>> {
    let certfile = File::open(path).context("cannot open certificate file")?;
    let mut reader = BufReader::new(certfile);
    let certs = certs(&mut reader)
        .context("cannot read certs")?
        .into_iter()
        .map(Certificate)
        .collect();
    Ok(certs)
}

fn load_keys(path: &str) -> Result<PrivateKey> {
    let keyfile = File::open(path).context("cannot open private key file")?;
    let mut reader = BufReader::new(keyfile);
    // Tenta di leggere le chiavi PKCS#8.
    let keys = rustls_pemfile::pkcs8_private_keys(&mut reader)
        .context("cannot read PKCS#8 private keys")?;

    // Controlla se è stata trovata almeno una chiave.
    if let Some(key) = keys.into_iter().next() {
        Ok(PrivateKey(key))
    } else {
        Err(anyhow::anyhow!("no PKCS#8 keys found"))
    }
}

fn main() -> Result<()> {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let certs = load_certs("certs/cert.pem").unwrap();
        let key = load_keys("certs/key.pem").unwrap();

        let server_crypto = rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .unwrap();

        let server_config = ServerConfig::with_crypto(Arc::new(server_crypto));

        let endpoint = Endpoint::server(server_config, "127.0.0.1:4433".parse()?).unwrap();

        println!("Server listening on {}", endpoint.local_addr().unwrap());

        // Accept incoming connections
        while let Some(connecting) = endpoint.accept().await {
            tokio::spawn(handle_connection(connecting.await?));
        }

        Ok(())
    })
}

async fn handle_connection(mut new_conn: quinn::Connection) -> Result<()> {
    println!("New connection: {}", new_conn.remote_address());

    // Handle incoming bi-directional streams
    let (send, recv) = new_conn
        .open_bi()
        .await?;

    tokio::spawn(handle_stream(send, recv));

    Ok(())
}


async fn handle_stream(mut send: quinn::SendStream, mut recv: quinn::RecvStream) -> Result<()> {
    // Creazione di un intervallo di un secondo
    let mut interval = time::interval(Duration::from_secs(1));

    loop {
        interval.tick().await; // Aspetta il prossimo tick dell'intervallo
        let message = "Hello from server!\n";


        // Prova a inviare il messaggio
        // if let Err(e) = send.write_all(message.as_bytes()).await {
        //     println!("Error sending message: {}", e);
        //     break;
        // }

        // send image
        let img = ImageReader::open("./tmp/test.png")?.decode()?;
        println!("{:?}", img.as_bytes());

        if let Err(e) = send.write_all(img.as_bytes()).await {
            println!("Error sending message: {}", e);
            break;
        }

        println!("Sendede");

        // Verifica se il client ha chiuso la connessione
        break;

    }
    send.finish().await?;
    println!("Stream finished");
    Ok(())
}


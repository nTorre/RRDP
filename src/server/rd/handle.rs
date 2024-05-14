use std::io::Cursor;
use std::sync::{Arc};
use std::time::Duration;
use image::{DynamicImage, ImageBuffer};
use image::ImageFormat::Jpeg;
use log::info;
use tokio::runtime::Runtime;
use crate::errors::connection_errors::ConnectionError;
use crate::handlers::handle_connection;
use quinn::{Connection, Endpoint, RecvStream, SendStream, ServerConfig as QuinnServerConfig};
use rustls::ServerConfig;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{Semaphore, Mutex};
use tokio::time::interval;
use x11rb::connection::Connection as XConnection;
use x11rb::protocol::xproto::{ConnectionExt, CreateWindowAux, ImageFormat, Screen, WindowClass};
use x11rb::rust_connection::RustConnection;

// TODO: implementare invio con i thread. Ogni volta che posso invio un'immagine,
// TODO: ma dev'essere acquisita in maniera concorrente

// async{faccio screen, blocco, scrivo buffer, sblocco}
// async{blocco, mando buf, sblocco}

// To implement
pub struct RrdpServer {

}

pub fn start_listening(server_crypto: ServerConfig) -> Result<(), ConnectionError> {

    let rt = match Runtime::new(){
        Ok(rt) => rt,
        Err(_) => return Err(ConnectionError::RunTimeError())
    };

    rt.block_on(async {
        let address = "0.0.0.0:4433"; // sistemare perché lo devo prendere dal file config
        let server_config = QuinnServerConfig::with_crypto(Arc::new(server_crypto));
        let parsed_address = match address.parse() {
            Ok(parsed_address) => parsed_address,
            Err(_) => return Err(ConnectionError::AddressParsingError(address.to_string()))
        };

        let endpoint = Endpoint::server(server_config, parsed_address).unwrap();
        info!("Server listening on {}", endpoint.local_addr().unwrap());
        // Accept incoming connections
        while let Some(connecting) = endpoint.accept().await {
            let connection = match connecting.await {
                Ok(connection) => connection,
                Err(e) => {return Err(
                    ConnectionError::CreateConnectionError(e.to_string()))}
            };
            tokio::spawn(handle_client(connection));
        }
        Ok(())
    })
}

async fn handle_client(connection: Connection) -> Result<(), ConnectionError> {
    let (mut send, recv) = match connection.open_bi().await{
        Ok((send, recv)) => (send, recv),
        Err(err) => {return Err(ConnectionError::OpenBiError(err.to_string()))}
    };

    // send standard info like screen dimension and other settings like number of frames

    // login phase
    login(&send, &recv).await;

    // if login is true, connect to display


    // sending images and receive commands
    return match send_image(&mut send).await {
        Ok(_) => {
            Ok(())
        }
        Err(err) => {
            Err(err)
        }
    };
}

async fn login(send: &SendStream, recv: &RecvStream){
    println!("{}", send.id());
}

pub async fn send_image(mut send: &mut SendStream) -> Result<(), ConnectionError> {
    let mut i = 0;
    let (conn, screen_num)  = x11rb::connect(None).unwrap();
    let screen = &conn.setup().roots[screen_num];

    let _ = conn.flush();

    loop {

        // Save the screenshot to a file
        let img = DynamicImage::from(get_image(&conn, screen).await.unwrap());

        if let Err(e) = send.write_all(img.as_bytes()).await {
            return Err(ConnectionError::SendingError(e.to_string()))
        }
        println!("Sent img {}", i);
        i+=1;

    }
}

async fn send_image_async(send: &mut SendStream) -> Result<(), ConnectionError> {
    let img_data = Arc::new(Mutex::new(DynamicImage::new_rgba8(800, 600)));

    let mut img_data_clone = img_data.clone();
    tokio::spawn(async move {
        let (conn, screen_num) = x11rb::connect(None).expect("Failed to connect to X server");
        let screen = &conn.setup().roots[screen_num];
        let mut interval = interval(Duration::from_millis(10));

        loop {
            let img = get_image(&conn, screen).await.unwrap();
            *img_data_clone.lock().await = img;
            interval.tick().await;
        }
    });

    loop {
        send.write_all(img_data.lock().await.as_bytes()).await;
    }

}



async fn get_image(conn: &RustConnection, screen: &Screen) -> Result<DynamicImage, String>{
    let image = conn.get_image(
        ImageFormat::Z_PIXMAP,
        screen.root,
        0,
        0,
        1024,
        768,
        !0,
    ).unwrap().reply().unwrap();

    let buffer = image.data;
    let img: ImageBuffer<image::Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(screen.width_in_pixels as u32, screen.height_in_pixels as u32, buffer).unwrap();
    let img = DynamicImage::from(img);

    Ok(img)
}

use std::borrow::Cow;
use std::error::Error;
use std::io::{Cursor, IoSlice};
use std::sync::{Arc};
use std::time::{Duration, Instant};
use image::{DynamicImage, EncodableLayout, ImageBuffer};
use image::ImageFormat::Png;
use log::info;
use tokio::runtime::Runtime;
use crate::errors::connection_errors::ConnectionError;
use crate::handlers::handle_connection;
use quinn::{Connection, Endpoint, RecvStream, SendStream, ServerConfig as QuinnServerConfig};
use rustls::internal::msgs::codec::u24;
use rustls::ServerConfig;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{Semaphore, Mutex, Notify};
use tokio::time::interval;
use x11rb::connection::{Connection as XConnection, RequestConnection};
use x11rb::cookie::VoidCookie;
use x11rb::protocol::xproto::{BUTTON_PRESS_EVENT, ButtonPressEvent, ButtonReleaseEvent, ChangeWindowAttributesAux, ConnectionExt, CreateWindowAux, EventMask, GetInputFocusReply, GrabMode, GrabStatus, ImageFormat, KeyButMask, QueryPointerReply, Screen, send_event, SendEventRequest, Time, Timestamp, Window, WindowClass};
use x11rb::rust_connection::RustConnection;
use x11rb::x11_utils::Serialize;
use chrono::Utc;
use x11rb::protocol::xproto;
use x11rb::protocol::xtest::{self, ConnectionExt as XTestExt};

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

pub async fn send_image_backup(mut send: &mut SendStream) -> Result<(), ConnectionError> {
    let mut i = 0;
    let (conn, screen_num)  = x11rb::connect(None).unwrap();
    let screen = &conn.setup().roots[screen_num];


    let _ = conn.flush();
    let mut fps_time = Instant::now();

    loop {
        let start = Instant::now();
        let img = get_image(&conn, screen).await.unwrap();

        // println!("Image taken in {:?}", start.elapsed());
        let start = Instant::now();

        if let Err(e) = send.write_all(img.as_bytes()).await {
            return Err(ConnectionError::SendingError(e.to_string()))
        }
        // println!("Sent img {}", i);
        // println!("Image sent in {:?}", start.elapsed());

        i+=1;

        if i==10{
            let secs = fps_time.elapsed().as_millis() as f64 / 1000.0;
            println!("fps: {}", i as f64/secs);
            fps_time = Instant::now();
            i=0;
        }
    }
}

async fn send_image(send: &mut SendStream) -> Result<(), ConnectionError> {
    let img_data = Arc::new(Mutex::new(DynamicImage::new_rgba8(800, 600)));
    let notify = Arc::new(Notify::new());

    let img_data_clone = img_data.clone();
    let notify_clone = notify.clone();

    // TODO: l'interval dovrebbe essere modificato di modo tale che ogni screen viene inviato
    // in questo caso probabilmente vengono fatti più screen di quelli inviati

    tokio::spawn(async move {
        let (conn, screen_num) = x11rb::connect(None).unwrap();
        let screen = &conn.setup().roots[screen_num];
        let mut interval = interval(Duration::from_millis(5));


        loop {
            interval.tick().await;
            let img = get_image(&conn, screen).await.unwrap();
            let mut img_guard = img_data_clone.lock().await;
            *img_guard = DynamicImage::from(img);
            drop(img_guard);
            println!("[TAKEN] Screen taken");
            notify_clone.notify_one();
        }
    });

    tokio::spawn(async move {
        let (conn, screen_num) = x11rb::connect(None).unwrap();
        let screen = &conn.setup().roots[screen_num];
        match conn.change_window_attributes(
            screen.root,
            &ChangeWindowAttributesAux::new().event_mask(EventMask::BUTTON_PRESS | EventMask::BUTTON_RELEASE | EventMask::POINTER_MOTION)
        ){
            Ok(_) => {}
            Err(e) => {println!("{}", e)}
        };

        let start_positions: Vec<(i16, i16)> = vec![
            (80, 330),
            (80, 330),
        ];

        for (x, y) in start_positions {

            conn.warp_pointer(
                x11rb::NONE,  // src_window
                screen.root,  // dst_window
                x, y,         // src_x, src_y
                screen.width_in_pixels, screen.height_in_pixels,         // src_width, src_height
                x, y          // dst_x, dst_y
            );


            // Sposta il puntatore
            println!("Moved to {},{}", x,y);
            conn.flush();
            // Aspetta un secondo prima di spostarti di nuovo
            std::thread::sleep(std::time::Duration::from_millis(1000));

            let root_window = screen.root;

            let xtest_version = conn.xtest_get_version(2,2).unwrap().reply().unwrap();
            println!("XTEST version: {}.{}", xtest_version.major_version, xtest_version.minor_version);

            // Simula un click del mouse (es. click sinistro)
            match simulate_mouse_click2(&conn, root_window, 1, x, y){
                Ok(_) => {}
                Err(e) => {println!("{}", e)}
            };


            std::thread::sleep(std::time::Duration::from_millis(10));

            match simulate_mouse_click2(&conn, root_window, 1, x, y){
                Ok(_) => {}
                Err(e) => {println!("{}", e)}
            };

        }
    });

    loop {
        notify.notified().await;
        let img_guard = img_data.lock().await;
        let data = <DynamicImage as Clone>::clone(&img_guard);
        send.write_all(data.as_bytes()).await;
        println!("[SENT] Screen sent");
    }
}


fn simulate_mouse_click2(conn: &RustConnection, window: u32, button: u8, x: i16, y: i16) -> Result<(), Box<dyn std::error::Error>> {
    // Simula la pressione del pulsante
    conn.xtest_fake_input(4, button, 0, window, x, y, 0)?;

    // Simula il rilascio del pulsante
    conn.xtest_fake_input(5, button, 0, window, 0, 0, 0)?;

    // Assicurati che i comandi siano stati inviati
    conn.flush()?;

    println!("Mouse click simulated.");

    Ok(())
}


async fn get_image(conn: &RustConnection, screen: &Screen) -> Result<image::RgbaImage, String>{
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
    let img = image::RgbaImage::from(img);

    Ok(img)
}

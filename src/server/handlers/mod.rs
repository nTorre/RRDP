use std::time::Duration;
use quinn::Connection;
use image::io::Reader as ImageReader;
use image::{DynamicImage, ImageBuffer};
use x11rb::connection::Connection as XConnection;
use x11rb::protocol::xproto::{ConnectionExt, CreateWindowAux, ImageFormat, WindowClass};

pub async fn simple_handler(connection: Connection) -> Result<(), ()>{
    println!("[server] connesso nuovo client - ip: {:?}", connection.remote_address());
    Ok(())
}

pub async fn handle_connection(mut new_conn: Connection) -> anyhow::Result<()> {
    println!("New connection: {}", new_conn.remote_address());

    // Handle incoming bidirectional streams
    let (send, recv) = new_conn
        .open_bi()
        .await?;

    tokio::spawn(send_image(send, recv));

    Ok(())
}

pub(crate) async fn get_image() -> Result<DynamicImage, String>{
    let res  = x11rb::connect(None);
    match res {
        Ok((conn, screen_num)) => {
            let screen = &conn.setup().roots[screen_num];
            let win_id = conn.generate_id().unwrap();
            conn.create_window(
                24,
                win_id,
                screen.root,
                0,
                0,
                1024,
                768,
                0,
                WindowClass::INPUT_OUTPUT,
                screen.root_visual,
                &CreateWindowAux::new().background_pixel(screen.white_pixel),
            ).unwrap();

            conn.map_window(win_id).unwrap();
            let _ = conn.flush();
            // Take a screenshot
            let image = conn.get_image(
                ImageFormat::Z_PIXMAP,
                screen.root,
                0,
                0,
                1024,
                768,
                !0,
            ).unwrap().reply().unwrap();

            // Save the screenshot to a file
            let buffer = image.data;
            let image: ImageBuffer<image::Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(screen.width_in_pixels as u32, screen.height_in_pixels as u32, buffer).unwrap();
            return Ok(DynamicImage::from(image))

        }
        Err(e) => {
            println!("{:?}", e);
            return Err(e.to_string());
        }
    }

}

fn get_image2(path: &str)->Result<DynamicImage, String>{
    let img = ImageReader::open(path);
    return match img {
        Ok(_) => {
            match img.unwrap().decode() {
                Ok(img) => {
                    Ok(img)
                },
                Err(e) => Err(format!("Errore nel caricamento dell'immagine: {:?}", e)),
            }
        }
        Err(e) => Err(format!("Errore nel caricamento dell'immagine: {:?}", e)),
    }
}

pub async fn send_image(mut send: quinn::SendStream, mut recv: quinn::RecvStream) -> anyhow::Result<()> {

    let mut interval = tokio::time::interval(Duration::from_secs(3));

    loop {
        interval.tick().await; // Aspetta il prossimo tick dell'intervallo


        // Prova a inviare il messaggio
        let path = "tmp/test.png";  // Sostituisci con il percorso corretto dell'immagine
        match get_image().await {
            Ok(img) =>{
                println!("{:?}", img.as_bytes().len());
                if let Err(e) = send.write_all(img.as_bytes()).await {
                    println!("Error sending message: {}", e);
                }
            },
            Err(e)=>{

            }
        }

        println!("Sendede");
        // Verifica se il client ha chiuso la connessione

    }
    send.finish().await?;
    println!("Stream finished");
    Ok(())
}


pub async fn ping(mut send: quinn::SendStream, mut recv: quinn::RecvStream) -> anyhow::Result<()> {
    // Creazione di un intervallo di un secondo
    let mut interval = tokio::time::interval(Duration::from_secs(1));

    loop {
        interval.tick().await; // Aspetta il prossimo tick dell'intervallo
        let message = "Hello from server!\n";


        // Prova a inviare il messaggio
        if let Err(e) = send.write_all(message.as_bytes()).await {
            println!("Error sending message: {}", e);
            break;
        }

        println!("Sendede");

        // Verifica se il client ha chiuso la connessione

    }
    send.finish().await?;
    println!("Stream finished");
    Ok(())
}

